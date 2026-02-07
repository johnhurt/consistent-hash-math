const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');
const { experiments } = require("webpack");

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  experiments: {
    asyncWebAssembly: true
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        {
          from: "index.html",
          to: "index.html",
        },
        {
          from: "assets/*",
          to: ".",
        },
      ],
    })
  ],
  module: {
    rules: [
      { test: /\.md$/, use: 'raw-loader' },
      {
        test: /\.css$/,
        use: ['style-loader', 'css-loader'], // style-loader comes first (last in the array)
      }
    ]
  },
};
