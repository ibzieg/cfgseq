
function checkClockRes(n, maxSteps) {

    let prev = 0;
    for (let i = 1.0; i <= maxSteps; i += 1.0) {
        const v = n / i;
        const d = Math.abs(v - prev);
        if (d < 2.0) {
            return false;
        }
        prev = v;
    }
    return true;
}

(function main() {
    const ppq = 24;
    const ppb = ppq * 4.0;
    let maxSteps = 64;

    let m;
    for (m = 1.0; m < 1000; m += 1.0) {
        let n = m * ppb;
        if (checkClockRes(n, maxSteps)) {
            console.log(`m=${m} is enough resolution per tick to allow maxSteps=${maxSteps}`);
            return;
        }
    }
    console.log(`failed to find solution after m=${m} attempts`);

})()