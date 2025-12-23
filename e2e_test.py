"""
E2E Test Script for Disk Dashboard
Uses pyautogui for automation
"""
import pyautogui
import time
import sys
import os

# Safety settings
pyautogui.FAILSAFE = True
pyautogui.PAUSE = 0.3

SCREENSHOT_DIR = os.path.dirname(os.path.abspath(__file__))

def screenshot(name, region=None):
    """Take a screenshot and save it"""
    # Refocus before screenshot
    try:
        import pygetwindow as gw
        windows = gw.getWindowsWithTitle('Disk')
        if windows:
            try:
                windows[0].activate()
            except:
                pass
            time.sleep(0.3)
    except:
        pass

    path = os.path.join(SCREENSHOT_DIR, f"test_{name}.png")
    if region:
        # Capture only the specified region (left, top, width, height)
        pyautogui.screenshot(path, region=region)
    else:
        pyautogui.screenshot(path)
    print(f"Screenshot saved: {path}")
    return path

# Global to store window region
APP_REGION = None

def focus_app():
    """Find and focus the Disk Dashboard window"""
    global APP_REGION
    try:
        import pygetwindow as gw
        windows = gw.getWindowsWithTitle('Disk')
        if not windows:
            windows = gw.getWindowsWithTitle('Dashboard')
        if not windows:
            windows = gw.getWindowsWithTitle('disk-dashboard')

        if windows:
            win = windows[0]
            print(f"Found window: '{win.title}'")
            try:
                win.activate()
            except:
                pass  # Ignore activation errors
            # Don't maximize - test at default window size
            time.sleep(0.5)
            # Re-fetch window position after maximize
            windows = gw.getWindowsWithTitle('Disk')
            if windows:
                win = windows[0]
            # Adjust for window borders (negative coords mean maximized)
            left = max(0, win.left)
            top = max(0, win.top)
            width = win.width
            height = win.height
            APP_REGION = (left, top, width, height)
            return left, top, width, height
        else:
            print("Available windows:")
            for w in gw.getAllWindows():
                if w.title:
                    print(f"  - {w.title}")
    except Exception as e:
        print(f"Error finding window: {e}")
    return None

def refocus_window():
    """Re-focus the app window before each action"""
    try:
        import pygetwindow as gw
        windows = gw.getWindowsWithTitle('Disk')
        if windows:
            try:
                windows[0].activate()
            except:
                pass
            time.sleep(0.2)
    except:
        pass

def click_at(x, y, description=""):
    """Click at specific coordinates"""
    refocus_window()
    pyautogui.click(x, y)
    print(f"Clicked at ({x}, {y}) - {description}")
    time.sleep(0.3)

def run_test():
    """Main test sequence"""
    print("=" * 50)
    print("E2E Test for Disk Dashboard")
    print("=" * 50)

    # Focus the app window
    print("\nFocusing Disk Dashboard window...")
    win_info = focus_app()

    if not win_info:
        print("ERROR: Could not find Disk Dashboard window!")
        print("Make sure the app is running.")
        return False

    left, top, width, height = win_info
    print(f"Window at: ({left}, {top}) size: {width}x{height}")

    # Calculate DYNAMIC positions based on window dimensions
    # Left panel is ~25% of width, right panel is ~75%
    left_panel_width = int(width * 0.25)
    left_panel_center = left + left_panel_width // 2

    right_panel_left = left + left_panel_width
    right_panel_width = width - left_panel_width
    right_panel_center = right_panel_left + right_panel_width // 2

    # Header is ~15% from top, content starts ~25% from top
    header_y = top + int(height * 0.15)
    content_start_y = top + int(height * 0.30)
    row_height = int(height * 0.06)  # ~6% per row

    # Column positions (from right edge): Use ~85%, ~92% of right panel
    cat_col_x = right_panel_left + int(right_panel_width * 0.75)
    size_col_x = right_panel_left + int(right_panel_width * 0.85)

    time.sleep(0.5)

    # Test 1: Take initial screenshot
    print("\n[Test 1] Initial state")
    screenshot("01_initial", APP_REGION)

    # Test 2: Click on first disk (at ~25% height in left panel)
    print("\n[Test 2] Click on C: drive")
    disk_y = top + int(height * 0.25)
    click_at(left_panel_center, disk_y, "C: drive card")
    time.sleep(1)
    screenshot("02_after_disk_click", APP_REGION)

    # Test 3: Check if file browser appeared, click on a folder
    print("\n[Test 3] Click on first item in file list")
    click_at(right_panel_center, content_start_y + row_height, "First file/folder")
    time.sleep(0.5)
    screenshot("03_after_item_click", APP_REGION)

    # Test 4: Click Cat column header to test sorting
    print("\n[Test 4] Click Cat column to sort")
    click_at(cat_col_x, header_y, "Cat column header")
    time.sleep(0.3)
    screenshot("04_after_cat_sort", APP_REGION)

    # Test 5: Click Size column
    print("\n[Test 5] Click Size column to sort")
    click_at(size_col_x, header_y, "Size column header")
    time.sleep(0.3)
    screenshot("05_after_size_sort", APP_REGION)

    # Test 6: Test hover effects
    print("\n[Test 6] Testing hover effects")
    for i in range(4):
        hover_y = content_start_y + (i * row_height)
        pyautogui.moveTo(right_panel_center, hover_y)
        time.sleep(0.2)
    screenshot("06_hover_test", APP_REGION)

    # Test 7: Navigate back using .. button (at ~22% height)
    print("\n[Test 7] Click back/parent button")
    back_btn_y = top + int(height * 0.22)
    click_at(right_panel_left + 50, back_btn_y, "Back button")
    time.sleep(0.5)
    screenshot("07_after_back", APP_REGION)

    print("\n" + "=" * 50)
    print("E2E Test Complete!")
    print("=" * 50)
    return True

if __name__ == "__main__":
    print("Starting E2E test in 2 seconds...")
    time.sleep(2)
    success = run_test()
    sys.exit(0 if success else 1)
