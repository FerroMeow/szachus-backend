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

document.addEventListener("szachus-init", szachusInit);

if (typeof window.localStorage?.getItem("jwt") === "string") {
  document.dispatchEvent(new Event("szachus-init"));
} else {
  const form = document.getElementById("login-form");
  form.style.removeProperty("display");
  form.addEventListener("submit", onLogin);
}
