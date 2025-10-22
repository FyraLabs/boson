// Hook to re-register the greenworks module

console.log("BOSON: Hooking Greenworks");

console.debug("Attempting to override NAPI module path");

// const { pathToFileURL } = require("node:url");
const overrideRequire = require("override-require");

// https://github.com/ElectronForConstruct/greenworks-prebuilds/releases/download/v0.8.0/greenworks-electron-v125-linux-x64.node

// We are going to do the hook twice

// First hook pass: Replace the NAPI module path, so that it points to the correct NAPI module for the current Electron ABI

function getElectronAbi() {
    const absoluteElectronPath = process.execPath;
    // try to run electron -a and get output
    const { execSync } = require("node:child_process");
    const electronAbi = execSync(`${absoluteElectronPath} -a`)
        .toString()
        .trim();
    console.debug("Electron ABI:", electronAbi);
    return electronAbi;
}
let napi;
const napiDir = __dirname + "/lib/napi";

let napi_filename = `greenworks-electron-v${getElectronAbi()}-linux-x64.node`;

let napiPath = `${napiDir}/${napi_filename}`;
// let napiPath = `${napiDir}/greenworks-electron-v125-linux-x64.node`;

// strip .node extension
let napiPathNoExt = napiPath.replace(/\.node$/, "");

function http_get(url) {
    // recursive function that follows redirects
    const http = require("node:https");
    const fs = require("node:fs");

    const file = fs.createWriteStream(napiPath);

    const request = http.get(url, function (response) {
        if (
            response.statusCode >= 300 &&
            response.statusCode < 400 &&
            response.headers.location
        ) {
            console.log("Following redirect to:", response.headers.location);
            file.close();
            fs.unlinkSync(napiPath); // Clean up partial file
            http_get(response.headers.location);
            return;
        } else if (response.statusCode !== 200) {
            console.error(
                `Failed to download NAPI file: HTTP ${response.statusCode}`,
            );
            file.close();
            fs.unlinkSync(napiPath); // Clean up partial file
            return;
        } else if (
            response.headers["content-type"] &&
            response.headers["content-type"].includes("text/html")
        ) {
            console.error(
                "Failed to download NAPI file: Server returned HTML (likely 404 page)",
            );
            file.close();
            fs.unlinkSync(napiPath); // Clean up partial file
            return;
        } else {
            response.pipe(file);
        }

        file.on("finish", function () {
            file.close();
            console.log("Download complete.");
            console.log(
                "You may want to restart the game before Greenworks will work.",
            );
            napi = require(napiPathNoExt);
        });
    });

    request.on("error", function (err) {
        console.error("Failed to download NAPI file:", err.message);
        file.close();
        if (fs.existsSync(napiPath)) {
            fs.unlinkSync(napiPath); // Clean up partial file
        }
    });
}

function attempt_download_napi() {
    const http = require("node:https");
    const fs = require("node:fs");

    // check if file already exists
    fs.mkdirSync(napiDir, { recursive: true });
    if (fs.existsSync(napiPath)) {
        console.log("NAPI file already exists. Skipping download.");
        return;
    }

    try {
        // recursively create the directory

        const url = `https://github.com/ElectronForConstruct/greenworks-prebuilds/releases/download/v0.10.0/${napi_filename}`;

        const file = fs.createWriteStream(napiPath);

        const request = http_get(url);
    } catch (e) {
        console.error("Failed to download NAPI file:", e);
        exit(1);
    }
}

attempt_download_napi();

try {
    console.log("Loading", napiPathNoExt);
    napi = require(napiPathNoExt);
    // console.log("Loaded NAPI module:", napi);
} catch (e) {
    console.error(e);
}

const earlyOverride = (request, parent) => {
    // console.debug("PASS 1 CHECKING:", request);
    return request.includes("lib/greenworks-linux64");
};

const earlyResolve = (request, parent) => {
    console.debug("PASS 1 REQUEST:", request);
    return napi;
};

// Early override pass

overrideRequire(earlyOverride, earlyResolve);

let greenworks;
try {
    greenworks = require(__dirname + "/lib/greenworks");
} catch (e) {
    console.error(e);
}

// console.log("Greenworks:", greenworks);

// === END FIRST PASS ===

// Second hook pass: Replace the actual game import for greenworks, so that it points to our custom greenworks module

const isOverride = (request) => {
    console.debug("Trying to load", request);

    if (
        request.includes("greenworks/greenworks") ||
        request.includes("steamworks.js")
    ) {
        console.debug("OVERRIDE:", request);
        return true;
    }
    // return request.includes("steamworks.js");
    return false;
};

const resolveRequest = (request) => {
    console.debug("Parsing Request:", request);

    if (request.includes("steamworks.js")) {
        console.log(
            "Returning steamworks.js loader",
            __dirname + "/lib/steamworksjs",
        );
        return require(__dirname + "/lib/steamworksjs");
    }

    if (request.includes("greenworks/greenworks")) {
        console.log("Returning Greenworks module", greenworks);
        return greenworks;
    }

    // return greenworks;
};

overrideRequire(isOverride, resolveRequest);
