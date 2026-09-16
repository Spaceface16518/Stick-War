/** Fixed-width xorshift: JSON stores the seed as decimal text, never a Number. */
export class CombatRandom {
  private state: bigint;
  constructor(seed: string) {
    this.state = BigInt(seed);
  }
  next(): number {
    let x = this.state;
    x ^= x << 13n;
    x ^= x >> 7n;
    x ^= x << 17n;
    this.state = BigInt.asUintN(64, x);
    return Number(this.state >> 11n) / 9007199254740992;
  }
  signed(): number {
    return this.next() * 2 - 1;
  }
}
