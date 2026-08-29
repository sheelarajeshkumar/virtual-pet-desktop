(function (root) {
  "use strict";

  const REQUIRED_ANIMATIONS = ["idle", "walk", "sleep"];
  const ID_PATTERN = /^[a-z0-9]+(?:[.-][a-z0-9]+)+$/;
  const ANIMATION_PATTERN = /^[a-z][a-z0-9-]*$/;

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

  function normalizeAtlas(value, manifest, field = "atlas") {
    const source = object(value, field);
    return Object.freeze({
      src: assetUrl(source.file, manifest, `${field}.file`),
      columns: positiveInteger(source.columns, `${field}.columns`),
      rows: positiveInteger(source.rows, `${field}.rows`),
      frameWidth: positiveInteger(source.frameWidth, `${field}.frameWidth`),
      frameHeight: positiveInteger(source.frameHeight, `${field}.frameHeight`),
    });
  }

  function normalizeAnimations(value, atlas, field = "animations", required = []) {
    const source = object(value, field);
    const animations = Object.fromEntries(
      Object.entries(source).map(([name, value]) => {
        if (!ANIMATION_PATTERN.test(name)) {
          throw new TypeError(`${field} contains invalid animation name ${JSON.stringify(name)}`);
        }
        const animation = object(value, `${field}.${name}`);
        if (!Number.isInteger(animation.row) || animation.row < 0 || animation.row >= atlas.rows) {
          throw new TypeError(`${field}.${name}.row must reference an atlas row`);
        }
        if (
          !Array.isArray(animation.frames) ||
          !animation.frames.length ||
          animation.frames.some(
            (frame) => !Number.isInteger(frame) || frame < 0 || frame >= atlas.columns,
          )
        ) {
          throw new TypeError(`${field}.${name}.frames must reference atlas columns`);
        }
        if (typeof animation.loop !== "boolean") {
          throw new TypeError(`${field}.${name}.loop must be a boolean`);
        }
        return [
          name,
          Object.freeze({
            row: animation.row,
            frames: Object.freeze([...animation.frames]),
            frameDurationMs: positiveInteger(
              animation.frameDurationMs,
              `${field}.${name}.frameDurationMs`,
            ),
            loop: animation.loop,
          }),
        ];
      }),
    );
    for (const name of required) {
      if (!animations[name]) throw new TypeError(`${field}.${name} is required`);
    }
    return Object.freeze(animations);
  }

  function normalizeDirectional(value, manifest) {
    if (value === undefined) return Object.freeze({});
    const source = object(value, "directional");
    const directional = {};

    for (const [groupName, groupValue] of Object.entries(source)) {
      if (!["horizontal", "vertical", "diagonal"].includes(groupName)) {
        throw new TypeError(`directional.${groupName} is not supported`);
      }
      const field = `directional.${groupName}`;
      const group = object(groupValue, field);
      const atlas = normalizeAtlas(group.atlas, manifest, `${field}.atlas`);
      if (groupName === "horizontal") {
        directional[groupName] = Object.freeze({
          atlas,
          animations: normalizeAnimations(group.animations, atlas, `${field}.animations`),
        });
        continue;
      }

      const directions = object(group.animations, `${field}.animations`);
      const animations = {};
      for (const direction of ["up", "down"]) {
        if (directions[direction] !== undefined) {
          animations[direction] = normalizeAnimations(
            directions[direction],
            atlas,
            `${field}.animations.${direction}`,
          );
        }
      }
      if (!animations.up && !animations.down) {
        throw new TypeError(`${field}.animations must define up or down`);
      }
      for (const direction of Object.keys(directions)) {
        if (!["up", "down"].includes(direction)) {
          throw new TypeError(`${field}.animations.${direction} is not supported`);
        }
      }
      directional[groupName] = Object.freeze({ atlas, animations: Object.freeze(animations) });
    }
    return Object.freeze(directional);
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
    const animations = normalizeAnimations(
      source.animations,
      atlas,
      "animations",
      REQUIRED_ANIMATIONS,
    );

    return Object.freeze({
      schemaVersion: positiveInteger(source.schemaVersion ?? 1, "schemaVersion"),
      id,
      name: text(source.name, "name"),
      version: text(source.version ?? "1.0.0", "version"),
      species: text(source.species, "species"),
      manifestUrl: resolvedManifestUrl.href,
      atlas,
      animations,
      directional: normalizeDirectional(source.directional, resolvedManifestUrl),
      sounds: normalizeSounds(source.sounds, resolvedManifestUrl),
    });
  }

  function resolveAnimation(pack, behavior, travelDirection = "horizontal") {
    const base = { atlas: pack.atlas, animation: pack.animations[behavior] ?? pack.animations.idle };
    const groupName = travelDirection.includes("diagonal")
      ? "diagonal"
      : travelDirection === "up" || travelDirection === "down"
        ? "vertical"
        : "horizontal";
    const group = pack.directional?.[groupName];
    if (!group) return base;
    const animation =
      groupName === "horizontal"
        ? group.animations[behavior]
        : group.animations[travelDirection.startsWith("up") ? "up" : "down"]?.[behavior];
    return animation ? { atlas: group.atlas, animation } : base;
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

  const api = Object.freeze({
    REQUIRED_ANIMATIONS,
    normalizeCatalog,
    normalize,
    resolveAnimation,
    load,
  });
  root.VirtualPetPack = api;
  if (typeof module === "object" && module.exports) module.exports = api;
})(typeof window === "object" ? window : globalThis);
