import { moduleTwo } from './module2';

console.log("Basic project");

(async function() {
    const image = import('./empty.png');

    const moduleOne = await import('./module1');
    moduleOne();

    moduleTwo()
})();