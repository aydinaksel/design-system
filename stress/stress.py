import os
import random
import sys
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor

from playwright.sync_api import Error as PlaywrightError
from playwright.sync_api import sync_playwright

ROUTES = [
    "/",
    "/tables",
    "/tables?mode=cards",
    "/tables?search=gran",
    "/client-table",
    "/files",
    "/tables?mode=calendar",
    "/sidebar/dashboard",
]
VIEWPORTS = [{"width": 1280, "height": 900}, {"width": 390, "height": 844}]
SEQUENTIAL_REQUESTS_PER_ROUTE = 20
CONCURRENT_REQUESTS_PER_ROUTE = 40
REQUEST_TIMEOUT_SECONDS = 10
RANDOM_ACTIONS_PER_PAGE = 150
ACTION_TIMEOUT_MILLISECONDS = 300
PAGE_LOAD_TIMEOUT_MILLISECONDS = 15000
SETTLE_MILLISECONDS = 30
RECENT_ACTIONS_SHOWN = 8
IGNORED_CONSOLE_ERRORS = ["Failed to load resource"]
TYPED_TEXT = ["gran", "", "a" * 300, "<script>", "😀 ünïcödé", "0", "-1"]
KEYS = ["Tab", "Shift+Tab", "Escape", "Enter", " ", "ArrowDown", "ArrowUp", "ArrowLeft", "ArrowRight", "Home", "End"]

INTERACTIVE_SELECTOR = "button:not([disabled]), a[href], input:not([type=file]):not([disabled]), textarea, [role=combobox], [tabindex='0']"

CANDIDATES_SCRIPT = """([selector, origin]) => [...document.querySelectorAll(selector)].map((element, index) => ({element, index})).filter(({element}) => {
    const box = element.getBoundingClientRect();
    const visible = box.width > 0 && box.height > 0 && getComputedStyle(element).visibility !== 'hidden';
    const leavesGallery = element.tagName === 'A' && (
        element.hasAttribute('download') || element.target === '_blank' || new URL(element.href, location.href).origin !== origin
    );
    return visible && !leavesGallery;
}).map(({index}) => index)"""

DROP_SCRIPT = """(index) => {
    const target = document.querySelectorAll('*')[index] || document.body;
    const transfer = new DataTransfer();
    transfer.items.add(new File([new Uint8Array([137, 80, 78, 71])], 'stress.png', {type: 'image/png'}));
    for (const type of ['dragenter', 'dragover', 'drop']) {
        target.dispatchEvent(new DragEvent(type, {dataTransfer: transfer, bubbles: true, cancelable: true}));
    }
}"""


def status_of(url):
    try:
        with urllib.request.urlopen(url, timeout=REQUEST_TIMEOUT_SECONDS) as response:
            response.read()
            return response.status
    except urllib.error.HTTPError as error:
        return error.code
    except OSError as error:
        return repr(error)


def hammer(base_url):
    failures = []
    for route in ROUTES:
        print(f"  hammering {route}", flush=True)
        url = base_url + route
        statuses = [status_of(url) for _ in range(SEQUENTIAL_REQUESTS_PER_ROUTE)]
        with ThreadPoolExecutor(max_workers=CONCURRENT_REQUESTS_PER_ROUTE) as pool:
            statuses += list(pool.map(status_of, [url] * CONCURRENT_REQUESTS_PER_ROUTE))
        bad = [status for status in statuses if status != 200]
        if bad:
            failures.append(f"{route}: {len(bad)} of {len(statuses)} requests failed, e.g. {bad[0]}")
    return failures


def collect_errors(page):
    errors = []
    page.on("pageerror", lambda error: errors.append(f"page error: {error}"))
    page.on(
        "console",
        lambda message: message.type == "error"
        and not any(ignored in message.text for ignored in IGNORED_CONSOLE_ERRORS)
        and errors.append(f"console error: {message.text}"),
    )
    return errors


def open_page(browser, base_url, route, viewport):
    page = browser.new_page(viewport=viewport)
    errors = collect_errors(page)
    try:
        page.goto(base_url + route, wait_until="networkidle", timeout=PAGE_LOAD_TIMEOUT_MILLISECONDS)
    except PlaywrightError:
        errors.append("the page did not finish loading")
    return page, errors


def hydration_sweep(browser, base_url):
    failures = []
    for viewport in VIEWPORTS:
        for route in ROUTES:
            print(f"  loading {route} at {viewport['width']}px", flush=True)
            page, errors = open_page(browser, base_url, route, viewport)
            page.wait_for_timeout(300)
            failures += [f"{route} at {viewport['width']}px: {error}" for error in errors]
            page.close()
    return failures


def random_action(page, chooser, origin):
    candidates = page.evaluate(CANDIDATES_SCRIPT, [INTERACTIVE_SELECTOR, origin])
    kind = chooser.choice(["click", "click", "click", "type", "key", "key", "drop"])
    if kind == "key":
        key = chooser.choice(KEYS)
        page.keyboard.press(key)
        return f"press {key}"
    if kind == "drop":
        index = chooser.randrange(0, 2000)
        page.evaluate(DROP_SCRIPT, index)
        return f"drop a file on element {index}"
    if not candidates:
        return "nothing to act on"
    index = chooser.choice(candidates)
    element = page.locator(INTERACTIVE_SELECTOR).nth(index)
    description = element.evaluate("element => element.outerHTML.slice(0, 80)")
    if kind == "type" and element.evaluate("element => ['INPUT', 'TEXTAREA'].includes(element.tagName)"):
        text = chooser.choice(TYPED_TEXT)
        element.fill(text, timeout=ACTION_TIMEOUT_MILLISECONDS)
        return f"type {text[:20]!r} into {description}"
    element.click(timeout=ACTION_TIMEOUT_MILLISECONDS)
    return f"click {description}"


def random_interaction(browser, base_url, seed):
    failures = []
    origin = base_url.rstrip("/")
    for viewport in VIEWPORTS:
        for route in ROUTES:
            print(f"  {RANDOM_ACTIONS_PER_PAGE} random actions on {route} at {viewport['width']}px", flush=True)
            chooser = random.Random(f"{seed}{route}{viewport['width']}")
            page, errors = open_page(browser, base_url, route, viewport)
            recent = []
            for _ in range(RANDOM_ACTIONS_PER_PAGE):
                try:
                    recent.append(random_action(page, chooser, origin))
                except PlaywrightError:
                    recent.append("action could not be performed")
                page.wait_for_timeout(SETTLE_MILLISECONDS)
                if errors:
                    break
            if errors:
                steps = "\n      ".join(recent[-RECENT_ACTIONS_SHOWN:])
                failures.append(f"{route} at {viewport['width']}px: {errors[0]}\n    last actions:\n      {steps}")
            page.close()
    return failures


def main():
    base_url = sys.argv[1]
    seed = sys.argv[2]
    chromium_path = os.environ.get("STRESS_CHROMIUM")
    if not chromium_path:
        sys.exit("STRESS_CHROMIUM is not set: run this inside the dev shell on Linux.")
    print(f"stress seed {seed}", flush=True)
    print("server hammering", flush=True)
    failures = [f"server: {failure}" for failure in hammer(base_url)]
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(executable_path=chromium_path, args=["--no-sandbox"])
        print("fresh page loads", flush=True)
        failures += [f"hydration: {failure}" for failure in hydration_sweep(browser, base_url)]
        print("random interaction", flush=True)
        failures += [f"interaction: {failure}" for failure in random_interaction(browser, base_url, seed)]
        browser.close()
    for failure in failures:
        print(f"FAIL {failure}")
    if failures:
        sys.exit(f"{len(failures)} stress failures with seed {seed}")
    print("stress passed")


if __name__ == "__main__":
    main()
