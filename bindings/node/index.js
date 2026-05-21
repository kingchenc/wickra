/* eslint-disable */
/* prettier-ignore */
// Platform-aware loader for the Wickra Node native binding.
//
// In production (after `npm install wickra`) the actual `.node` binary
// lives inside a per-platform subpackage that npm installs as an
// optional dependency — e.g. `wickra-linux-x64-gnu`. In development
// (after `napi build --platform`) the binary sits next to this file.
// We try the local path first and fall back to the subpackage so the
// same loader works in both modes.

const { platform, arch } = process;
const { join } = require('node:path');
const { existsSync } = require('node:fs');

const TARGETS = {
  'linux-x64': 'linux-x64-gnu',
  'darwin-x64': 'darwin-x64',
  'darwin-arm64': 'darwin-arm64',
  'win32-x64': 'win32-x64-msvc',
};

function load() {
  const key = `${platform}-${arch}`;
  const target = TARGETS[key];
  if (!target) {
    throw new Error(
      `wickra: this platform/architecture combination is not supported (${key}). ` +
        'Open an issue at https://github.com/kingchenc/wickra/issues with your platform details.'
    );
  }

  const localBinary = join(__dirname, `wickra.${target}.node`);
  if (existsSync(localBinary)) {
    return require(localBinary);
  }

  try {
    return require(`wickra-${target}`);
  } catch (err) {
    throw new Error(
      `wickra: failed to load the native binding for ${key}. ` +
        `Expected either ${localBinary} or the wickra-${target} package to be installed. ` +
        `Run \`npm install\` again, or build from source with \`npm run build\`. ` +
        `Underlying error: ${err.message}`
    );
  }
}

module.exports = load();
