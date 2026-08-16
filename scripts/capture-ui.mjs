import { spawn } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const baseUrl = process.env.AGENT_PIPELINE_REVIEW_URL ?? "http://127.0.0.1:1420";
const chromePath = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const reviewDir = resolve(".impeccable/review");
const profileDir = await mkdtemp(join(tmpdir(), "agent-pipeline-review-"));
const port = 9337;

await mkdir(reviewDir, { recursive: true });
const chrome = spawn(chromePath, [
  "--headless=new",
  `--remote-debugging-port=${port}`,
  `--user-data-dir=${profileDir}`,
  "--hide-scrollbars",
  "--disable-gpu",
  "--window-size=1230,768",
  baseUrl,
], { stdio: "ignore" });

const pause = (milliseconds = 220) => new Promise((resolvePause) => setTimeout(resolvePause, milliseconds));

async function json(path) {
  const response = await fetch(`http://127.0.0.1:${port}${path}`);
  if (!response.ok) throw new Error(`Chrome DevTools ${path}: ${response.status}`);
  return response.json();
}

for (let attempt = 0; attempt < 80; attempt += 1) {
  try {
    await json("/json/version");
    break;
  } catch {
    if (attempt === 79) throw new Error("Chrome DevTools did not start");
    await pause(100);
  }
}

const target = (await json("/json/list")).find((entry) => entry.type === "page");
if (!target) throw new Error("No Chrome page target found");
const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolveOpen, rejectOpen) => {
  socket.addEventListener("open", resolveOpen, { once: true });
  socket.addEventListener("error", rejectOpen, { once: true });
});

let messageId = 0;
const pending = new Map();
socket.addEventListener("message", (event) => {
  const message = JSON.parse(event.data);
  if (!message.id) return;
  const waiter = pending.get(message.id);
  if (!waiter) return;
  pending.delete(message.id);
  if (message.error) waiter.reject(new Error(message.error.message));
  else waiter.resolve(message.result);
});

function call(method, params = {}) {
  const id = ++messageId;
  socket.send(JSON.stringify({ id, method, params }));
  return new Promise((resolveCall, rejectCall) => pending.set(id, { resolve: resolveCall, reject: rejectCall }));
}

async function evaluate(expression) {
  const result = await call("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
  if (result.exceptionDetails) throw new Error(result.exceptionDetails.text);
  return result.result.value;
}

async function clickButton(text, exact = false) {
  const clicked = await evaluate(`(() => {
    const label = ${JSON.stringify(text)};
    const element = [...document.querySelectorAll('button')].find((button) =>
      ${exact ? "button.textContent.trim() === label" : "button.textContent.includes(label)"}
    );
    if (!element) return false;
    element.click();
    return true;
  })()`);
  if (!clicked) throw new Error(`Button not found: ${text}`);
  await pause();
}

async function clickAria(label) {
  const clicked = await evaluate(`(() => {
    const element = document.querySelector('[aria-label=${JSON.stringify(label)}]');
    if (!element) return false;
    element.click();
    return true;
  })()`);
  if (!clicked) throw new Error(`ARIA target not found: ${label}`);
  await pause();
}

async function capture(name) {
  await pause(350);
  const result = await call("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  await writeFile(join(reviewDir, name), Buffer.from(result.data, "base64"));
}

try {
  await call("Page.enable");
  await call("Runtime.enable");
  await call("Emulation.setDeviceMetricsOverride", { width: 1230, height: 768, deviceScaleFactor: 1, mobile: false });
  await pause(700);

  await evaluate("localStorage.clear(); location.reload()");
  await pause(900);
  await capture("onboarding-privacy.png");
  await clickButton("继续", true);
  await clickButton("继续", true);
  await capture("onboarding-doctor.png");
  await clickButton("继续", true);
  await clickButton("继续", true);
  await clickButton("进入 Mission Control");
  await capture("desktop.png");
  await capture("hero-repro.png");
  await writeFile(resolve("docs/images/mission-control.png"), Buffer.from((await call("Page.captureScreenshot", { format: "png", captureBeyondViewport: false })).data, "base64"));

  await clickAria("进入 Node Focus");
  await capture("node-focus.png");
  await clickAria("退出 Node Focus");
  await clickButton("请求修改");
  await capture("review-feedback-attempt-2.png");

  await clickButton("Deliverables");
  await capture("deliverables.png");
  await clickButton("Create Pipeline");
  await clickButton("生成 Package Proposal");
  await capture("create-pipeline.png");

  await clickButton("Graph", true);
  await clickButton("System");
  await clickButton("Warm Paper");
  await capture("theme-warm.png");
  await clickButton("Warm Paper");
  await clickButton("Night Ops");
  await capture("theme-night.png");

  await clickButton("模拟完成当前工作");
  await clickButton("批准并继续");
  await clickButton("模拟完成当前工作");
  await clickButton("模拟完成当前工作");
  await capture("completed-night.png");
} finally {
  socket.close();
  chrome.kill("SIGTERM");
  await pause(500);
  await rm(profileDir, { recursive: true, force: true }).catch(() => {});
}

console.log(`Captured Agent Pipeline review states in ${reviewDir}`);
