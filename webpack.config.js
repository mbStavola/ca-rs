var webpack = require('webpack');
var path = require('path');

var BUILD_DIR = path.join(__dirname, 'src/frontend/build');
var APP_DIR = path.join(__dirname, 'src/frontend/app');

var config = {
  devtool: 'cheap-module-source-map',
  entry: APP_DIR + '/app.js',
  output: {
    path: BUILD_DIR,
    filename: 'bundle.js',
    sourceMapFilename: "bundle.js.map",
  }
};

module.exports = config;