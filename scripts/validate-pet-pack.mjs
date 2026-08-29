#!/usr/bin/env node

import assert from "node:assert/strict";
import { readFile, realpath, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REQUIRED_ANIMATIONS = ["idle", "walk", "run", "sleep"];
const AUDIO_EXTENSIONS = new Set([".wav", ".mp3", ".ogg", ".m4a"]);

function object(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function text(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function safeRelativePath(value) {
  return (
    text(value) &&
    !path.posix.isAbsolute(value) &&
    !/^[A-Za-z]:/.test(value) &&
    !value.includes("\\") &&
    !value.includes("\0") &&
    value.split("/").every((part) => part !== "" && part !== "." && part !== "..") &&
    path.posix.normalize(value) === value
  );
}

function validateManifest(manifest) {
  const errors = [];
  const requireValue = (condition, message) => {
    if (!condition) errors.push(message);
  };

  requireValue(object(manifest), "manifest must be a JSON object");
  if (!object(manifest)) return errors;

  requireValue(manifest.schemaVersion === 1, "schemaVersion must be 1");
  requireValue(
    typeof manifest.id === "string" && /^[a-z0-9]+(?:[.-][a-z0-9]+)+$/.test(manifest.id),
    "id must be a lowercase reverse-domain-style identifier",
  );
  requireValue(text(manifest.name), "name must be a non-empty string");
  requireValue(
    typeof manifest.version === "string" && /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(manifest.version),
    "version must be a semantic version",
  );
  requireValue(
    typeof manifest.species === "string" && /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(manifest.species),
    "species must be a lowercase identifier",
  );
  requireValue(text(manifest.description), "description must be a non-empty string");
  requireValue(
    Array.isArray(manifest.authors) &&
      manifest.authors.length > 0 &&
      manifest.authors.every((author) => object(author) && text(author.name)),
    "authors must contain at least one author with a name",
  );

  const atlas = manifest.atlas;
  requireValue(object(atlas), "atlas must be an object");
  if (object(atlas)) {
    requireValue(safeRelativePath(atlas.file), "atlas.file must be a safe relative path");
    requireValue(path.posix.extname(atlas.file ?? "").toLowerCase() === ".png", "atlas.file must be a PNG");
    for (const field of ["columns", "rows", "frameWidth", "frameHeight"]) {
      requireValue(Number.isInteger(atlas[field]) && atlas[field] > 0, `atlas.${field} must be a positive integer`);
    }
  }

  requireValue(object(manifest.animations), "animations must be an object");
  if (object(manifest.animations) && object(atlas)) {
    for (const required of REQUIRED_ANIMATIONS) {
      requireValue(object(manifest.animations[required]), `animations.${required} is required`);
    }
    for (const [name, animation] of Object.entries(manifest.animations)) {
      requireValue(/^[a-z][a-z0-9-]*$/.test(name), `animation name ${JSON.stringify(name)} is invalid`);
      if (!object(animation)) {
        errors.push(`animations.${name} must be an object`);
        continue;
      }
      requireValue(
        Number.isInteger(animation.row) && animation.row >= 0 && animation.row < atlas.rows,
        `animations.${name}.row must reference an atlas row`,
      );
      requireValue(
        Array.isArray(animation.frames) &&
          animation.frames.length > 0 &&
          animation.frames.every((frame) => Number.isInteger(frame) && frame >= 0 && frame < atlas.columns),
        `animations.${name}.frames must reference atlas columns`,
      );
      requireValue(
        Number.isInteger(animation.frameDurationMs) &&
          animation.frameDurationMs >= 40 &&
          animation.frameDurationMs <= 60_000,
        `animations.${name}.frameDurationMs must be an integer from 40 to 60000`,
      );
      requireValue(typeof animation.loop === "boolean", `animations.${name}.loop must be Boolean`);
    }
  }

  if (manifest.sounds !== undefined) {
    requireValue(object(manifest.sounds), "sounds must be an object");
    if (object(manifest.sounds)) {
      for (const [name, sound] of Object.entries(manifest.sounds)) {
        requireValue(/^[a-z][a-z0-9-]*$/.test(name), `sound name ${JSON.stringify(name)} is invalid`);
        if (!object(sound)) {
          errors.push(`sounds.${name} must be an object`);
          continue;
        }
        requireValue(safeRelativePath(sound.file), `sounds.${name}.file must be a safe relative path`);
        requireValue(
          AUDIO_EXTENSIONS.has(path.posix.extname(sound.file ?? "").toLowerCase()),
          `sounds.${name}.file must use a supported audio extension`,
        );
        requireValue(
          typeof sound.volume === "number" && sound.volume >= 0 && sound.volume <= 1,
          `sounds.${name}.volume must be between 0 and 1`,
        );
        requireValue(typeof sound.loop === "boolean", `sounds.${name}.loop must be Boolean`);
      }
    }
  }

  requireValue(object(manifest.license), "license must be an object");
  if (object(manifest.license)) {
    requireValue(text(manifest.license.spdx), "license.spdx must be a non-empty SPDX identifier");
    requireValue(Array.isArray(manifest.license.assets), "license.assets must be an array");
    if (Array.isArray(manifest.license.assets)) {
      const seen = new Set();
      for (const [index, asset] of manifest.license.assets.entries()) {
        const prefix = `license.assets[${index}]`;
        if (!object(asset)) {
          errors.push(`${prefix} must be an object`);
          continue;
        }
        requireValue(safeRelativePath(asset.path), `${prefix}.path must be a safe relative path`);
        requireValue(text(asset.creator), `${prefix}.creator must be a non-empty string`);
        requireValue(text(asset.source), `${prefix}.source must be a non-empty string`);
        requireValue(text(asset.spdx), `${prefix}.spdx must be a non-empty SPDX identifier`);
        requireValue(text(asset.modifications), `${prefix}.modifications must be a non-empty string`);
        if (safeRelativePath(asset.path)) {
          requireValue(!seen.has(asset.path), `${prefix}.path duplicates ${asset.path}`);
          seen.add(asset.path);
        }
      }
      const referenced = [
        ...(safeRelativePath(atlas?.file) ? [atlas.file] : []),
        ...(object(manifest.sounds)
          ? Object.values(manifest.sounds)
              .filter((sound) => object(sound) && safeRelativePath(sound.file))
              .map((sound) => sound.file)
          : []),
      ];
      for (const assetPath of referenced) {
        requireValue(seen.has(assetPath), `license.assets must document ${assetPath}`);
      }
    }
  }

  return errors;
}

async function validateAsset(packRoot, relativePath) {
  const resolvedRoot = await realpath(packRoot);
  const candidate = path.resolve(resolvedRoot, relativePath);
  const resolvedAsset = await realpath(candidate);
  const relative = path.relative(resolvedRoot, resolvedAsset);
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    throw new Error(`${relativePath} resolves outside the pack directory`);
  }
  if (!(await stat(resolvedAsset)).isFile()) throw new Error(`${relativePath} is not a regular file`);
  return resolvedAsset;
}

async function pngDimensions(filename) {
  const bytes = await readFile(filename);
  const signature = "89504e470d0a1a0a";
  if (bytes.length < 24 || bytes.subarray(0, 8).toString("hex") !== signature) {
    throw new Error("atlas is not a valid PNG");
  }
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
}

async function validatePack(packDirectory) {
  const root = path.resolve(packDirectory);
  let manifest;
  try {
    manifest = JSON.parse(await readFile(path.join(root, "pet-pack.json"), "utf8"));
  } catch (error) {
    throw new Error(`cannot read pet-pack.json: ${error.message}`);
  }

  const errors = validateManifest(manifest);
  if (errors.length) throw new Error(errors.join("\n"));

  const atlasFile = await validateAsset(root, manifest.atlas.file);
  const dimensions = await pngDimensions(atlasFile);
  const expectedWidth = manifest.atlas.columns * manifest.atlas.frameWidth;
  const expectedHeight = manifest.atlas.rows * manifest.atlas.frameHeight;
  if (dimensions.width !== expectedWidth || dimensions.height !== expectedHeight) {
    throw new Error(
      `atlas is ${dimensions.width}x${dimensions.height}; expected ${expectedWidth}x${expectedHeight}`,
    );
  }

  for (const sound of Object.values(manifest.sounds ?? {})) await validateAsset(root, sound.file);
  for (const asset of manifest.license.assets) await validateAsset(root, asset.path);
  return manifest;
}

async function selfTest() {
  const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
  const puppy = path.resolve(scriptDirectory, "../src/pet-packs/puppy");
  const manifest = await validatePack(puppy);
  assert.equal(manifest.id, "virtual-pet-desktop.puppy");

  for (const unsafePath of [
    "../outside.png",
    "/outside.png",
    "C:/outside.png",
    "assets\\atlas.png",
  ]) {
    const unsafe = structuredClone(manifest);
    unsafe.atlas.file = unsafePath;
    assert(validateManifest(unsafe).some((error) => error.includes("safe relative path")));
  }

  const invalidFrame = structuredClone(manifest);
  invalidFrame.animations.walk.frames = [manifest.atlas.columns];
  assert(validateManifest(invalidFrame).some((error) => error.includes("atlas columns")));
  console.log("Pet pack validator self-test passed.");
}

async function main() {
  const argument = process.argv[2];
  if (argument === "--self-test") return selfTest();
  if (!argument || process.argv.length !== 3) {
    throw new Error("usage: node scripts/validate-pet-pack.mjs <pack-directory> | --self-test");
  }
  const manifest = await validatePack(argument);
  console.log(`Valid pet pack: ${manifest.name} (${manifest.id}@${manifest.version})`);
}

main().catch((error) => {
  console.error(`Pet pack validation failed:\n${error.message}`);
  process.exitCode = 1;
});
