#!/usr/bin/env node

// Required parameters:
// @raycast.schemaVersion 1
// @raycast.title Time Check
// @raycast.mode fullOutput

// Optional parameters:
// @raycast.icon ⏰

// Documentation:
// @raycast.description Check Time Doctor
// @raycast.author Kemal İlke Tuna

const https = require('https');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Load .env file
function loadEnv() {
    const envPath = path.join(__dirname, '.env');

    if (!fs.existsSync(envPath)) {
        console.error('❌ .env file not found!');
        console.log('Please create a .env file with:');
        console.log('email=your_email@example.com');
        console.log('password=your_password');
        process.exit(1);
    }

    const envContent = fs.readFileSync(envPath, 'utf-8');
    const envVars = {};

    envContent.split('\n').forEach(line => {
        line = line.trim();
        if (line && !line.startsWith('#')) {
            const [key, ...valueParts] = line.split('=');
            if (key && valueParts.length > 0) {
                envVars[key.trim()] = valueParts.join('=').trim();
            }
        }
    });

    return envVars;
}

// Load environment variables
const env = loadEnv();

// Parse boolean from env (accepts: true/1/yes or false/0/no)
function parseBoolean(value, defaultValue = false) {
    if (!value) return defaultValue;
    const normalized = value.toLowerCase().trim();
    return ['true', '1', 'yes'].includes(normalized);
}

// Parse contract periods from env
function parseContractPeriods(periodsString) {
    const periods = [];

    if (!periodsString) {
        console.error('❌ Missing contract_periods in .env file!');
        process.exit(1);
    }

    const entries = periodsString.split(',');

    for (const entry of entries) {
        const [dateStr, hoursStr] = entry.trim().split(':');

        if (!dateStr || !hoursStr) {
            console.error(`❌ Invalid contract period format: "${entry}"`);
            console.log('Expected format: YYYY-MM-DD:HOURS');
            process.exit(1);
        }

        // Parse date as local midnight (not UTC) to match getWeekStartMonday behavior
        const dateWithTime = dateStr.trim() + 'T00:00:00';
        const date = new Date(dateWithTime);
        const hours = parseFloat(hoursStr.trim());

        if (isNaN(date.getTime())) {
            console.error(`❌ Invalid date in contract period: "${dateStr}"`);
            process.exit(1);
        }

        if (isNaN(hours) || hours < 0) {
            console.error(`❌ Invalid hours in contract period: "${hoursStr}"`);
            process.exit(1);
        }

        periods.push({ date, hours });
    }

    // Sort by date (oldest first)
    periods.sort((a, b) => a.date - b.date);

    return periods;
}

// CLI flags
const cliSkipCurrentWeek = process.argv.includes('--skip-current-week');
const cliNoCache = process.argv.includes('--no-cache');

// Optional: reset cumulative balance from this date (week boundary = Monday of that week)
const resetCumulativeFromDate = env.reset_cumulative_from_date
    ? getWeekStartMonday(new Date(env.reset_cumulative_from_date.trim() + 'T00:00:00'))
    : null;

// Configuration
const CONFIG = {
    email: env.email,
    password: env.password,
    companyId: env.company_id,
    timezone: env.timezone,
    // Parse dates as local midnight (not UTC) to match getWeekStartMonday behavior
    startDate: new Date(env.start_date + 'T00:00:00'),
    skipCurrentWeek: cliSkipCurrentWeek || parseBoolean(env.skip_current_week, true),
    contractPeriods: parseContractPeriods(env.contract_periods),
    resetCumulativeFromDate
};

// Validate required env variables
const requiredVars = ['email', 'password', 'company_id', 'start_date', 'contract_periods'];
const missingVars = requiredVars.filter(varName => !env[varName]);

if (missingVars.length > 0) {
    console.error('❌ Missing required environment variables!');
    console.log('\nMissing:', missingVars.join(', '));
    console.log('\nPlease ensure your .env file contains:');
    console.log('email=your_email@example.com');
    console.log('password=your_password');
    console.log('company_id=your_company_id');
    console.log('timezone=Europe/Istanbul');
    console.log('start_date=2025-11-17');
    console.log('skip_current_week=true');
    console.log('contract_periods=2025-11-17:20,2026-02-02:28');
    process.exit(1);
}

// Helper: Parse cookies from Set-Cookie headers
function parseCookies(headers) {
    const cookies = {};
    const setCookieHeaders = headers['set-cookie'] || [];

    setCookieHeaders.forEach(cookie => {
        const parts = cookie.split(';')[0].split('=');
        const name = parts[0];
        const value = parts.slice(1).join('=');
        cookies[name] = value;
    });

    return cookies;
}

// Helper: Format cookies for Cookie header
function formatCookies(cookies) {
    return Object.entries(cookies)
        .map(([name, value]) => `${name}=${value}`)
        .join('; ');
}

// Helper: Make HTTPS request
function makeRequest(options, postData = null) {
    return new Promise((resolve, reject) => {
        const req = https.request(options, (res) => {
            let data = '';
            res.on('data', (chunk) => data += chunk);
            res.on('end', () => {
                try {
                    resolve({
                        statusCode: res.statusCode,
                        headers: res.headers,
                        body: JSON.parse(data)
                    });
                } catch (e) {
                    resolve({
                        statusCode: res.statusCode,
                        headers: res.headers,
                        body: data
                    });
                }
            });
        });
        req.on('error', reject);
        if (postData) req.write(postData);
        req.end();
    });
}

// Authenticate and get cookies
async function login() {
    const postData = JSON.stringify({
        email: CONFIG.email,
        password: CONFIG.password
    });

    const options = {
        hostname: 'api2.timedoctor.com',
        path: '/api/2.0/auth/v2/login',
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(postData),
            'Origin': 'https://2.timedoctor.com',
            'Referer': 'https://2.timedoctor.com/'
        }
    };

    const response = await makeRequest(options, postData);

    if (response.statusCode !== 200) {
        throw new Error(`Login failed: ${response.statusCode} - ${JSON.stringify(response.body)}`);
    }

    // Extract cookies from Set-Cookie headers
    const cookies = parseCookies(response.headers);

    if (!cookies['__Host-accessToken']) {
        throw new Error('No access token received from login');
    }

    return cookies;
}

// Get week stats from TimeDoctor API
async function getWeekStats(cookies, fromDate, toDate) {
    const queryParams = new URLSearchParams({
        from: fromDate.toISOString(),
        to: toDate.toISOString(),
        timezone: CONFIG.timezone,
        user: '',
        'group-by': 'company',
        fields: 'mobile,manual,offcomputer,computer,computerRatio,partial,total,paidBreak,unpaidBreak,paidLeave',
        untracked: '1',
        page: '0',
        limit: '200',
        company: CONFIG.companyId
    });

    const options = {
        hostname: 'api2.timedoctor.com',
        path: `/api/1.1/stats/total?${queryParams}`,
        method: 'GET',
        headers: {
            'Cookie': formatCookies(cookies),
            'Content-Type': 'application/json',
            'Origin': 'https://2.timedoctor.com',
            'Referer': 'https://2.timedoctor.com/'
        }
    };

    const response = await makeRequest(options);

    if (response.statusCode !== 200) {
        console.error('Stats API error:', response.statusCode, response.body);
        throw new Error(`Failed to get stats: ${response.statusCode}`);
    }

    return response.body;
}

// --- Cache for past weeks (macOS: ~/Library/Caches/time-checker) ---
function getCacheDir() {
    const dir = path.join(os.homedir(), 'Library', 'Caches', 'time-checker', CONFIG.companyId || 'default');
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
    return dir;
}

function weekCacheKey(monday) {
    const y = monday.getFullYear();
    const m = String(monday.getMonth() + 1).padStart(2, '0');
    const d = String(monday.getDate()).padStart(2, '0');
    return `${y}-${m}-${d}.json`;
}

function readWeekCache(monday) {
    try {
        const filePath = path.join(getCacheDir(), weekCacheKey(monday));
        if (!fs.existsSync(filePath)) return null;
        const raw = fs.readFileSync(filePath, 'utf-8');
        const data = JSON.parse(raw);
        if (!data || typeof data !== 'object') return null;
        return data;
    } catch {
        return null;
    }
}

function writeWeekCache(monday, stats) {
    try {
        const filePath = path.join(getCacheDir(), weekCacheKey(monday));
        fs.writeFileSync(filePath, JSON.stringify(stats), 'utf-8');
    } catch (e) {
        // Non-fatal: log and continue
        if (typeof console !== 'undefined' && console.error) {
            console.error(`[Cache] Write failed for ${weekCacheKey(monday)}:`, e.message);
        }
    }
}

// Get Monday of a given week
function getWeekStartMonday(date) {
    const d = new Date(date);
    const dayOfWeek = d.getDay();
    const daysToMonday = dayOfWeek === 0 ? -6 : 1 - dayOfWeek;

    d.setDate(d.getDate() + daysToMonday);
    d.setHours(0, 0, 0, 0);

    return d;
}

// Format date range for display (e.g., "Jan 1 - Jan 7, 2026")
function formatWeekRange(monday) {
    const sunday = new Date(monday);
    sunday.setDate(monday.getDate() + 6);

    const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

    return `${monthNames[monday.getMonth()]} ${monday.getDate()} - ${monthNames[sunday.getMonth()]} ${sunday.getDate()}, ${sunday.getFullYear()}`;
}

// Get the week ending date (Sunday)
function getWeekEndSunday(monday) {
    const sunday = new Date(monday);
    sunday.setDate(monday.getDate() + 6);
    sunday.setHours(23, 59, 59, 999);
    return sunday;
}

// Calculate target hours for a given week based on contract periods
function getTargetHours(monday) {
    // Find the applicable contract period (latest period where date <= monday)
    let applicableHours = CONFIG.contractPeriods[0].hours; // Default to first period

    for (const period of CONFIG.contractPeriods) {
        if (monday >= period.date) {
            applicableHours = period.hours;
        } else {
            break; // Periods are sorted, so we can stop here
        }
    }

    return applicableHours;
}

// Process requests in batches with rate limiting
async function batchProcess(tasks, batchSize = 5, delayMs = 500) {
    const results = [];

    for (let i = 0; i < tasks.length; i += batchSize) {
        const batch = tasks.slice(i, i + batchSize);

        try {
            const batchResults = await Promise.all(batch.map(task => task()));
            results.push(...batchResults);

            // Add delay between batches (except for the last batch)
            if (i + batchSize < tasks.length) {
                await new Promise(resolve => setTimeout(resolve, delayMs));
            }
        } catch (error) {
            console.error(`\x1b[31m[ERROR] Batch processing error: ${error.message}\x1b[0m`);
            // Continue with next batch even if one fails
        }
    }

    return results;
}

// Main analysis function
async function analyzeWorkHours() {
    console.log('🔐 Authenticating...\n');
    const cookies = await login();
    console.log('✅ Authenticated successfully\n');

    // Collect all weeks to fetch
    const weeksToFetch = [];
    let currentMonday = getWeekStartMonday(new Date());
    const today = new Date();
    const thisWeekMonday = getWeekStartMonday(today);

    console.log('📋 Preparing week list...\n');

    while (currentMonday >= CONFIG.startDate) {
        // SKIP: Current week if configured
        if (CONFIG.skipCurrentWeek && currentMonday >= thisWeekMonday) {
            const weekRange = formatWeekRange(currentMonday);
            console.log(`\x1b[90m[SKIP] ${weekRange} (current/future week)\x1b[0m`);
            currentMonday = new Date(currentMonday);
            currentMonday.setDate(currentMonday.getDate() - 7);
            continue;
        }

        weeksToFetch.push(new Date(currentMonday));
        currentMonday = new Date(currentMonday);
        currentMonday.setDate(currentMonday.getDate() - 7);
    }

    if (weeksToFetch.length === 0) {
        console.log('\x1b[33mNo weeks to fetch\x1b[0m\n');
        return;
    }

    console.log(`📊 Fetching ${weeksToFetch.length} weeks in parallel (batches of 10)...\n`);

    const buildWeekRow = (monday, weekRange, stats) => {
        const totalHours = (stats.data?.[0]?.total || 0) / 3600;
        const targetHours = getTargetHours(monday);
        const balance = totalHours - targetHours;
        return {
            monday,
            Week: weekRange,
            Worked: formatHours(totalHours),
            Target: `${targetHours}h`,
            Balance: `${balance >= 0 ? '+' : ''}${formatHours(balance)}`,
            rawBalance: balance,
        };
    };

    // Create tasks for parallel execution
    const fetchTasks = weeksToFetch.map(monday => async () => {
        const weekRange = formatWeekRange(monday);
        const sunday = getWeekEndSunday(monday);
        const isPastWeek = monday < thisWeekMonday;
        try {
            // Use cache for all past weeks (current week is never cached)
            if (isPastWeek && !cliNoCache) {
                const cached = readWeekCache(monday);
                if (cached) {
                    const row = buildWeekRow(monday, weekRange, cached);
                    return { ...row, fromCache: true };
                }
            }

            const stats = await getWeekStats(cookies, monday, sunday);
            if (isPastWeek) {
                writeWeekCache(monday, stats);
            }
            const row = buildWeekRow(monday, weekRange, stats);
            return { ...row, fromCache: false };
        } catch (error) {
            console.error(`\x1b[31m[ERROR] ${weekRange}: ${error.message}\x1b[0m`);
            return null;
        }
    });

    // Execute in batches with rate limiting
    const results = await batchProcess(fetchTasks, 10, 500);

    // Filter out failed requests and sort by date (newest first)
    const sortedResults = results
        .filter(result => result !== null)
        .sort((a, b) => b.monday - a.monday);

    // Print fetch results in sorted order
    for (const row of sortedResults) {
        const indicator = row.fromCache ? '[OK·]' : '[OK]';
        console.log(`\x1b[32m${indicator} ${row.Week} | ${row.Worked} | Target: ${row.Target}\x1b[0m`);
    }
    console.log('');

    // Cumulative is oldest→newest, compute on reversed array then map back
    const oldestFirst = [...sortedResults].reverse();
    const cumulativeByMonday = new Map();
    let runningSum = 0;
    for (const row of oldestFirst) {
        if (!CONFIG.resetCumulativeFromDate || row.monday >= CONFIG.resetCumulativeFromDate) {
            runningSum += row.rawBalance;
        }
        cumulativeByMonday.set(row.monday.getTime(), runningSum);
    }

    const weeklyData = sortedResults
        .map((row) => {
            const { monday, rawBalance, fromCache, ...rest } = row;
            const cumulative = cumulativeByMonday.get(monday.getTime());
            return {
                ...rest,
                Status: rawBalance >= 0 ? '✅' : '❌',
                Cumulative: `${cumulative >= 0 ? '+' : ''}${formatHours(cumulative)}`,
                'Cumul.': cumulative >= 0 ? '✅' : '❌',
                rawBalance,
                rawCumulative: cumulative
            };
        });

    if (CONFIG.resetCumulativeFromDate) {
        const r = CONFIG.resetCumulativeFromDate;
        const resetStr = `${r.getFullYear()}-${String(r.getMonth() + 1).padStart(2, '0')}-${String(r.getDate()).padStart(2, '0')}`;
        console.log(`\x1b[90mCumulative balance reset from ${resetStr} (week starting that Monday)\x1b[0m`);
    }

    // Display results
    displayResults(weeklyData);
}

// Format decimal hours as "Xh Ym" (e.g. 16.59 -> "16h 35m")
function formatHours(decimalHours) {
    const sign = decimalHours < 0 ? '-' : '';
    const abs = Math.abs(decimalHours);
    const h = Math.floor(abs);
    const m = Math.round((abs - h) * 60);
    if (m === 0) return `${sign}${h}h`;
    return `${sign}${h}h ${m}m`;
}

// Display formatted results with summary
function displayResults(weeklyData) {
    const tableWidth = 100;
    const separator = '='.repeat(tableWidth);
    const title = '📈  WORK HOURS ANALYSIS  📈';
    // emoji takes 2 chars visually, adjust padding (each emoji = 2 visual chars, 1 JS char extra)
    const titleVisualLen = title.length + 2; // 2 extra for 2 emoji
    const titlePadding = Math.max(0, Math.floor((tableWidth - titleVisualLen) / 2));
    const centeredTitle = ' '.repeat(titlePadding) + title;

    console.log('\n' + separator);
    console.log(centeredTitle);
    console.log(separator + '\n');

    const displayData = [...weeklyData].reverse();
    console.table(displayData.map(({ rawBalance, rawCumulative, ...rest }) => rest));

    // Total balance = most recent week's cumulative (index 0, sorted newest first)
    const totalBalance = weeklyData.length > 0
        ? weeklyData[0].rawCumulative
        : 0;
    const textColor = totalBalance >= 0 ? '\x1b[32m' : '\x1b[31m'; // Green or Red
    const bold = '\x1b[1m';
    const reset = '\x1b[0m';

    console.log(`\n${bold}${textColor}Total Balance: ${totalBalance >= 0 ? '+' : ''}${formatHours(totalBalance)}${reset}\n`);
}

// Run the script
analyzeWorkHours().catch(error => {
    console.error('\n❌ Error:', error.message);
    process.exit(1);
});
