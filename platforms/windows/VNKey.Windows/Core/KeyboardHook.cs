using System;
using System.Runtime.InteropServices;

namespace VNKey.Windows.Core
{
    public class KeyboardHook : IDisposable
    {
        private EngineWrapper _engine;
        // Keep a reference to prevent Garbage Collector from wiping the callback!
        private ToggleCallbackDelegate _callbackDelegate;
        private IntPtr _callbackPtr;

        public event Action<bool>? OnVietnameseModeChanged;

        public KeyboardHook(EngineWrapper engine)
        {
            _engine = engine;

            // 1. Prepare C# callback function mapping for Rust Native FFI to use
            _callbackDelegate = new ToggleCallbackDelegate(HandleToggleCallback);
            _callbackPtr = Marshal.GetFunctionPointerForDelegate(_callbackDelegate);
            EngineWrapper.vnkey_set_toggle_callback(_callbackPtr);

            // 2. Instruct the Native Core DLL to spawn a completely disconnected Background Thread
            //    which listens to keyboard hooks inside Windows natively, skipping C# GC entirely!
            EngineWrapper.vnkey_hook_start();
        }

        private void HandleToggleCallback(bool isEnabled)
        {
            // Ensure this event fires safely inside WPF Dispatcher if it interacts with UI
            OnVietnameseModeChanged?.Invoke(isEnabled);
        }

        public void SetVietnameseMode(bool enabled)
        {
            _engine.SetVietnameseMode(enabled);
            OnVietnameseModeChanged?.Invoke(enabled);
        }

        public void Dispose()
        {
            EngineWrapper.vnkey_hook_stop();
        }
    }
}
