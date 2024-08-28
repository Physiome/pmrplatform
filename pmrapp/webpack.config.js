const path = require('path');

module.exports = {
  entry: './src/index.js',
  experiments: {
    outputModule: true,
  },
  mode: 'production',
  output: {
    filename: 'pmrapp-bundle.js',
    path: path.resolve(__dirname, 'dist'),
    library: {
      type: 'modern-module',
    }
  },
};
