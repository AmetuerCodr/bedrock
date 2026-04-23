

const [score1, score2, score3]: [number, number, number] = [72, 73, 33];
const average = (score1 + score2 + score3) / 3;
console.log("Average: ", average.toFixed(1));


const hash = await Bun.password.hash("1234567890");

console.log(`hashed password: ${hash}`);