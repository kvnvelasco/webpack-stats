const path = require('path');

module.exports = {
    context: path.resolve(__dirname, "../common_src"),
    entry: {
        index: "./index.js",
        entryTwo: "./entry_two.js"
    },
    module: {
        rules: [
            {
                test: /\.png/,
                type: 'asset/resource'
            }
        ]
    },
    optimization: {
        splitChunks: {
            chunks: 'async',
            minChunks: 1,
            minSize: 1,

        }
    }
}