import {TestRunner} from 'klaver:test';


const runner = new TestRunner();

runner.describe("Describe 1", () => {
  runner.it('should', () => {
    
  })
  runner.describe("Describe 2", () => {
    runner.it('should inner', () => {
      
    })
  })
})

await runner.run();