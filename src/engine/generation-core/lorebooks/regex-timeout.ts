export function vmRegexExecutor(regex: RegExp, text: string): boolean {
  return regex.test(text);
}
