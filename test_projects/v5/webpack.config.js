const path = require('path');

module.exports = {
    context: path.resolve(__dirname, "../common_src"),
    entry: {
        index: "./index.js"
    },
    module: {
        rules: [
            {
                test: /\.png/,
                type: 'asset/resource'
            }
        ]
    }
}