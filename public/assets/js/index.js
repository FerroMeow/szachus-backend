import * as szachus from "../wasm/szachus/szachus.js";

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
  return claims.exp > current_timestamp_s;
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

let childObserver = new MutationObserver((mutations) => {
  for (const mutation of mutations) {
    let listOfAddedNodes = [...mutation.addedNodes];
    for (const node of listOfAddedNodes) {
      if (node.nodeName === "CANVAS") {
        let main = document.querySelector("main");
        let h3 = document.createElement("h3");
        h3.textContent = "Graj teraz";
        h3.classList.add("text-heading", "text-heading-3");
        main.append(h3);
        let container = document.createElement("div");
        container.classList.add("game-container");
        main.append(container);
        container.appendChild(document.querySelector("canvas[alt]"));
        childObserver.disconnect();
        return;
      }
    }
  }
});

childObserver.observe(document.body, {
  childList: true,
});

let jwt = window.localStorage?.getItem("jwt");

if (isPlayable(jwt)) {
  document.dispatchEvent(new Event("szachus-init"));
} else {
  const form = document.getElementById("login-form");
  form.style.removeProperty("display");
  form.addEventListener("submit", onLogin);
}
