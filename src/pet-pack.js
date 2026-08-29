(function (root) {
  "use strict";

  const REQUIRED_ANIMATIONS = ["idle", "walk", "sleep"];
  const ID_PATTERN = /^[a-z0-9]+(?:[.-][a-z0-9]+)+$/;

  function object(value, field) {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
      throw new TypeError(`${field} must be an object`);
    }
    return value;
  }

  function text(value, field) {
    if (typeof value !== "string" || !value.trim()) {
      throw new TypeError(`${field} must be a non-empty string`);
    }
    return value.trim();
  }

  function manifestUrl(url) {
    const base = root.document?.baseURI;
    try {
      return new URL(url, base);
    } catch {
      throw new TypeError("manifestUrl must be an absolute URL outside a browser");
    }
  }

  function assetUrl(path, manifest, field) {
    const value = text(path, field);
    let decoded;
    try {
      decoded = decodeURIComponent(value);
    } catch {
      throw new TypeError(`${field} contains invalid URL encoding`);
    }

    if (
      decoded.includes("\\") ||
      decoded.includes("\0") ||
      decoded.startsWith("/") ||
      decoded.split("/").includes("..") ||
      /^[a-z][a-z\d+.-]*:/i.test(decoded)
    ) {
      throw new TypeError(`${field} must be a safe path relative to the pet pack`);
    }

    const base = new URL("./", manifest);
    const resolved = new URL(value, base);
    if (
      resolved.protocol !== base.protocol ||
      resolved.host !== base.host ||
      !resolved.pathname.startsWith(base.pathname)
    ) {
      throw new TypeError(`${field} must stay inside the pet pack directory`);
    }
    return resolved.href;
  }

  function positiveInteger(value, field) {
    if (!Number.isInteger(value) || value < 1) {
      throw new TypeError(`${field} must be a positive integer`);
    }
    return value;
  }

  function normalizeAtlas(value, manifest) {
    const source = object(value, "atlas");
    return Object.freeze({
      src: assetUrl(source.file, manifest, "atlas.file"),
      columns: positiveInteger(source.columns, "atlas.columns"),
      rows: positiveInteger(source.rows, "atlas.rows"),
      frameWidth: positiveInteger(source.frameWidth, "atlas.frameWidth"),
      frameHeight: positiveInteger(source.frameHeight, "atlas.frameHeight"),
    });
  }

  function normalizeAnimations(value, atlas) {
    const source = object(value, "animations");
    const animations = Object.fromEntries(
      Object.entries(source).map(([name, value]) => {
        const animation = object(value, `animations.${name}`);
        if (!Number.isInteger(animation.row) || animation.row < 0 || animation.row >= atlas.rows) {
          throw new TypeError(`animations.${name}.row must reference an atlas row`);
        }
        if (
          !Array.isArray(animation.frames) ||
          !animation.frames.length ||
          animation.frames.some(
            (frame) => !Number.isInteger(frame) || frame < 0 || frame >= atlas.columns,
          )
        ) {
          throw new TypeError(`animations.${name}.frames must reference atlas columns`);
        }
        if (typeof animation.loop !== "boolean") {
          throw new TypeError(`animations.${name}.loop must be a boolean`);
        }
        return [
          name,
          Object.freeze({
            row: animation.row,
            frames: Object.freeze([...animation.frames]),
            frameDurationMs: positiveInteger(
              animation.frameDurationMs,
              `animations.${name}.frameDurationMs`,
            ),
            loop: animation.loop,
          }),
        ];
      }),
    );
    for (const name of REQUIRED_ANIMATIONS) {
      if (!animations[name]) throw new TypeError(`animations.${name} is required`);
    }
    return Object.freeze(animations);
  }

  function normalizeSounds(value, manifest) {
    if (value === undefined) return Object.freeze({});
    const source = object(value, "sounds");
    return Object.freeze(
      Object.fromEntries(
        Object.entries(source).map(([name, value]) => {
          const sound = object(value, `sounds.${name}`);
          if (typeof sound.volume !== "number" || sound.volume < 0 || sound.volume > 1) {
            throw new TypeError(`sounds.${name}.volume must be between 0 and 1`);
          }
          if (typeof sound.loop !== "boolean") {
            throw new TypeError(`sounds.${name}.loop must be a boolean`);
          }
          return [
            name,
            Object.freeze({
              src: assetUrl(sound.file, manifest, `sounds.${name}.file`),
              volume: sound.volume,
              loop: sound.loop,
            }),
          ];
        }),
      ),
    );
  }

  function normalize(rawManifest, url) {
    const source = object(rawManifest, "manifest");
    const resolvedManifestUrl = manifestUrl(url);
    const id = text(source.id, "id");
    if (!ID_PATTERN.test(id)) {
      throw new TypeError("id must use lowercase letters, numbers, and single hyphens");
    }

    const atlas = normalizeAtlas(source.atlas, resolvedManifestUrl);
    const animations = normalizeAnimations(source.animations, atlas);

    return Object.freeze({
      schemaVersion: positiveInteger(source.schemaVersion ?? 1, "schemaVersion"),
      id,
      name: text(source.name, "name"),
      version: text(source.version ?? "1.0.0", "version"),
      species: text(source.species, "species"),
      manifestUrl: resolvedManifestUrl.href,
      atlas,
      animations,
      sounds: normalizeSounds(source.sounds, resolvedManifestUrl),
    });
  }

  function normalizeCatalog(value) {
    if (!Array.isArray(value) || !value.length) {
      throw new TypeError("pet catalog must be a non-empty array");
    }
    const seen = new Set();
    return Object.freeze(
      value.map((entry, index) => {
        const source = object(entry, `catalog[${index}]`);
        const id = text(source.id, `catalog[${index}].id`);
        if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(id) || seen.has(id)) {
          throw new TypeError(`catalog[${index}].id must be a unique lowercase pet id`);
        }
        seen.add(id);
        const manifest = text(source.manifest, `catalog[${index}].manifest`);
        if (!/^pet-packs\/[a-z0-9-]+\/pet-pack\.json$/.test(manifest)) {
          throw new TypeError(`catalog[${index}].manifest must reference a bundled pet pack`);
        }
        return Object.freeze({ id, name: text(source.name, `catalog[${index}].name`), manifest });
      }),
    );
  }

  async function load(url, fetchImpl = root.fetch) {
    if (typeof fetchImpl !== "function") throw new TypeError("fetch is unavailable");
    const resolvedUrl = manifestUrl(url);
    const response = await fetchImpl(resolvedUrl.href);
    if (!response?.ok) {
      throw new Error(`Could not load pet pack (${response?.status ?? "no response"})`);
    }
    return normalize(await response.json(), resolvedUrl);
  }

  const api = Object.freeze({ REQUIRED_ANIMATIONS, normalizeCatalog, normalize, load });
  root.VirtualPetPack = api;
  if (typeof module === "object" && module.exports) module.exports = api;
})(typeof window === "object" ? window : globalThis);
