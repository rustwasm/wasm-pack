const { Binary } = require("binary-install");
const os = require("os");

const windows = "x86_64-pc-windows-msvc";

const getPlatform = () => {
  const type = os.type();
  const arch = os.arch();

  if (type === "Windows_NT" && arch === "x64") {
    return windows;
  }
  if (type === "Linux" && arch === "x64") {
    return "x86_64-unknown-linux-musl";
  }
  if (type === "Darwin" && (arch === "x64" || arch === "arm64")) {
    return "x86_64-apple-darwin";
  }

  throw new Error(`Unsupported platform: ${type} ${arch}`);
};

const getBinary = () => {
  const platform = getPlatform();
  const version = require("./package.json").version;
  const author = "rustwasm";
  const name = "wasm-pack";
  const url = `https://github.com/${author}/${name}/releases/download/v${version}/${name}-v${version}-${platform}.tar.gz`;
  return new Binary(platform === windows ? "wasm-pack.exe" : "wasm-pack", url);
};

const run = () => {
  const binary = getBinary();
  binary.run();
};

const install = () => {
  const binary = getBinary();
  binary.install();
};

const uninstall = () => {
  const binary = getBinary();
  binary.uninstall();
};

module.exports = {
  install,
  run,
  uninstall,
};
