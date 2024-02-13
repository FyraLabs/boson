// fetch GitHub assets for greenworks builds

// greenworks consts
const GREENWORKS_RELEASES: &str = "ElectronForConstruct/greenworks-prebuilds";
const GREENWORKS_PREFIX: &str = "greenworks-electron-v";
const GREENWORKS_PLATFORM: &str = "-linux-x64"; //.node
// install greenworks platform in $runtimedir/greenworks/$abi_ver/greenworks-linux64.node
// we can use octocrab to get the release for the specific ABI version
