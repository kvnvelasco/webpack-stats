

console.log("Basic project");

(function() {

    const moduleOne =  require(['./module1'], function(moduleOne) {
        moduleOne();
    });


    const {moduleTwo} = require(["./module2"], function(moduleTwo) {
        moduleTwo.moduleTwo()
    });

})();