const path = require('path');

module.exports = {
    context: path.resolve(__dirname, "../common_src"),
    entry: {
        index: "./index.js"
    },
    output: {
        path: path.resolve(__dirname, 'dist'),
      filename: "index.js"
    },
}