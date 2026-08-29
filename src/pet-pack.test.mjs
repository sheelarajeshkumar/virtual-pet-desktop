import assert from "node:assert/strict";
import test from "node:test";

await import("./pet-pack.js");
const { load, normalize, normalizeCatalog, resolveAnimation } = globalThis.VirtualPetPack;
const manifestUrl = "https://pets.example/packs/puppy/pet-pack.json";

function validManifest() {
  return {
    schemaVersion: 1,
    id: "example.little-puppy",
    name: "Little Puppy",
    species: "dog",
    atlas: {
      file: "assets/atlas.png",
      columns: 4,
      rows: 3,
      frameWidth: 128,
      frameHeight: 128,
    },
    animations: {
      idle: { row: 0, frames: [0, 1], frameDurationMs: 300, loop: true },
      walk: { row: 1, frames: [0, 1, 2, 3], frameDurationMs: 120, loop: true },
      sleep: { row: 2, frames: [0, 1], frameDurationMs: 500, loop: true },
    },
    sounds: { bark: { file: "sounds/bark.wav", volume: 0.6, loop: false } },
  };
}

test("normalizes defaults and resolves pack-relative assets", () => {
  const pack = normalize(validManifest(), manifestUrl);

  assert.equal(pack.version, "1.0.0");
  assert.equal(pack.species, "dog");
  assert.equal(pack.atlas.src, "https://pets.example/packs/puppy/assets/atlas.png");
  assert.deepEqual(pack.animations.walk.frames, [0, 1, 2, 3]);
  assert.equal(pack.sounds.bark.src, "https://pets.example/packs/puppy/sounds/bark.wav");
  assert.equal(pack.sounds.bark.loop, false);
});

test("resolves optional vertical and diagonal animation atlases", () => {
  const manifest = validManifest();
  manifest.directional = {
    horizontal: {
      atlas: { file: "assets/horizontal.png", columns: 4, rows: 1, frameWidth: 128, frameHeight: 128 },
      animations: {
        walk: { row: 0, frames: [3, 2, 1, 0], frameDurationMs: 110, loop: true },
      },
    },
    vertical: {
      atlas: { file: "assets/vertical.png", columns: 4, rows: 4, frameWidth: 128, frameHeight: 128 },
      animations: {
        down: { walk: { row: 0, frames: [0, 1], frameDurationMs: 120, loop: true } },
        up: { walk: { row: 2, frames: [2, 3], frameDurationMs: 120, loop: true } },
      },
    },
    diagonal: {
      atlas: { file: "assets/diagonal.png", columns: 4, rows: 2, frameWidth: 128, frameHeight: 128 },
      animations: {
        up: { run: { row: 1, frames: [0, 1, 2, 3], frameDurationMs: 80, loop: true } },
      },
    },
  };

  const pack = normalize(manifest, manifestUrl);
  assert.equal(resolveAnimation(pack, "walk", "horizontal").atlas.src, "https://pets.example/packs/puppy/assets/horizontal.png");
  assert.equal(resolveAnimation(pack, "walk", "up").atlas.src, "https://pets.example/packs/puppy/assets/vertical.png");
  assert.equal(resolveAnimation(pack, "walk", "up").animation.row, 2);
  assert.equal(resolveAnimation(pack, "run", "up-diagonal").animation.row, 1);
  assert.equal(resolveAnimation(pack, "sleep", "down").atlas, pack.atlas);
});

test("keeps legacy packs on their base atlas for every direction", () => {
  const pack = normalize(validManifest(), manifestUrl);
  assert.equal(resolveAnimation(pack, "walk", "up-diagonal").atlas, pack.atlas);
  assert.equal(resolveAnimation(pack, "walk", "up-diagonal").animation, pack.animations.walk);
});

test("rejects unsafe or malformed directional definitions", () => {
  const unsafe = validManifest();
  unsafe.directional = {
    vertical: {
      atlas: { file: "../vertical.png", columns: 4, rows: 1, frameWidth: 128, frameHeight: 128 },
      animations: { up: { walk: { row: 0, frames: [0], frameDurationMs: 120, loop: true } } },
    },
  };
  assert.throws(() => normalize(unsafe, manifestUrl), /safe path/);

  const invalidDirection = validManifest();
  invalidDirection.directional = {
    diagonal: {
      atlas: { file: "assets/diagonal.png", columns: 4, rows: 1, frameWidth: 128, frameHeight: 128 },
      animations: { sideways: { walk: { row: 0, frames: [0], frameDurationMs: 120, loop: true } } },
    },
  };
  assert.throws(() => normalize(invalidDirection, manifestUrl), /must define up or down/);
});

test("loads a manifest with the supplied fetch implementation", async () => {
  let requested;
  const fetch = async (url) => {
    requested = url;
    return { ok: true, json: async () => validManifest() };
  };

  const pack = await load(manifestUrl, fetch);
  assert.equal(requested, manifestUrl);
  assert.equal(pack.id, "example.little-puppy");
});

test("rejects assets outside the pack directory", () => {
  for (const unsafe of ["../secret.png", "/shared/pet.png", "https://evil.example/pet.png", "..%2fsecret.png", "folder\\pet.png"]) {
    const manifest = validManifest();
    manifest.atlas.file = unsafe;
    assert.throws(() => normalize(manifest, manifestUrl), /safe path|stay inside/);
  }
});

test("requires the core animations", () => {
  const manifest = validManifest();
  delete manifest.animations.sleep;
  assert.throws(() => normalize(manifest, manifestUrl), /animations\.sleep is required/);
});

test("rejects invalid manifests and failed fetches", async () => {
  assert.throws(() => normalize(null, manifestUrl), /manifest must be an object/);
  assert.throws(
    () => normalize({ ...validManifest(), id: "Bad ID" }, manifestUrl),
    /id must use lowercase/,
  );
  await assert.rejects(
    load(manifestUrl, async () => ({ ok: false, status: 404 })),
    /Could not load pet pack \(404\)/,
  );
});

test("catalog accepts bundled packs and rejects remote or duplicate entries", () => {
  const catalog = normalizeCatalog([
    { id: "puppy", name: "Puppy", manifest: "pet-packs/puppy/pet-pack.json" },
  ]);
  assert.equal(catalog[0].id, "puppy");
  assert.throws(
    () =>
      normalizeCatalog([
        { id: "bad", name: "Bad", manifest: "https://example.com/pet-pack.json" },
      ]),
    /bundled pet pack/,
  );
  assert.throws(
    () =>
      normalizeCatalog([
        { id: "cat", name: "Cat", manifest: "pet-packs/cat/pet-pack.json" },
        { id: "cat", name: "Other", manifest: "pet-packs/other/pet-pack.json" },
      ]),
    /unique lowercase pet id/,
  );
});
