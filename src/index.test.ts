import { execSync } from 'child_process';

describe('ts-hello-world', () => {
  it('should print "Hello, World!" to the console', () => {
    const output = execSync('npm start').toString().trim();
    expect(output).toBe('Hello, World!');
  });
});