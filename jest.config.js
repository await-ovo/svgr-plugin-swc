/** @type {import('jest').Config} */
module.exports = {
  testMatch: ['**/tests/**/*.test.ts'],
  transform: {
    "^.+\\.(t|j)sx?$": "@swc/jest",
  },
}