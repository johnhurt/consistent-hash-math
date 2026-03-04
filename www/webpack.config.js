const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');
const PrerendererWebpackPlugin = require('@prerenderer/webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin'); // Import the plugin

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
    new HtmlWebpackPlugin({
      template: path.resolve(__dirname, 'index.html'),
      filename: 'index.html'
    }),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: "assets/*",
          to: ".",
        },
      ],
    }),
    new PrerendererWebpackPlugin({
      staticDir: path.join(__dirname, 'dist'),
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
