import * as szachus from "./szachus-wasm/szachus.js";

async function szachusInit() {
  await szachus.default();
  szachus.main();
}

async function fetchJson(input, init) {
  const response = await fetch(input, init);
  if (!response.ok) {
    return null;
  }
  return response.json();
}

async function onLogin(e) {
  e.preventDefault();
  const formData = new FormData(e.target);
  const body = JSON.stringify({
    username: formData.get("username"),
    password: formData.get("password"),
  });
  let urlParams = {
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    method: "POST",
    body,
  };
  let result = await fetchJson("/user/login", urlParams);
  if (null === result) {
    result = await fetchJson("/user/register", urlParams);
  }
  if (typeof result?.jwt !== "string") {
    return;
  }
  localStorage.setItem("jwt", result.jwt);
  document.dispatchEvent(new Event("szachus-init"));
  const form = document.getElementById("login-form");
  form.style.display = "none";
  form.removeEventListener("submit", onLogin);
}

/**
 *
 * @param {string} jwtString
 * @returns {object|null}
 */
function parseJwtClaims(jwtString) {
  const claimsBase64Url = jwtString.split(".").at(1);
  if (claimsBase64Url === undefined) {
    return null;
  }
  const claimsBase64 = claimsBase64Url.replace(/-/g, "+").replace(/_/g, "/");
  try {
    return JSON.parse(window.atob(claimsBase64));
  } catch (error) {
    console.error(error);
    return null;
  }
}

/**
 *
 * @param {object} claims
 * @returns {boolean}
 */
function isClaimsExpirationValid(claims) {
  if (typeof claims.exp !== "number" || isNaN(claims.exp)) {
    return false;
  }
  let current_timestamp_ms = window.Date.now();
  let current_timestamp_s = Math.round(current_timestamp_ms * 0.001);
  console.debug(
    claims.exp,
    current_timestamp_s,
    claims.exp < current_timestamp_s
  );
  return claims.exp < current_timestamp_s;
}

/**
 *
 * @param {string|undefined} jwt
 * @returns {boolean}
 */
function isPlayable(jwt) {
  if (typeof jwt !== "string") {
    return false;
  }
  let claims = parseJwtClaims(jwt);
  if (claims === null) {
    return false;
  }
  return isClaimsExpirationValid(claims);
}

document.addEventListener("szachus-init", szachusInit);

let jwt = window.localStorage?.getItem("jwt");

if (isPlayable(jwt)) {
  document.dispatchEvent(new Event("szachus-init"));
} else {
  const form = document.getElementById("login-form");
  form.style.removeProperty("display");
  form.addEventListener("submit", onLogin);
}
