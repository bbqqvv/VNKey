using System;
using System.Runtime.InteropServices;
using VNKey.Windows.Engine;
using VNKey.Windows.Models;

namespace VNKey.Windows.Services
{
    public class EngineService : IEngineService, IDisposable
    {
        private readonly EngineWrapper _engine;
        private ToggleCallbackDelegate? _callbackDelegate;
        private OpenWindowCallbackDelegate? _openWindowCallbackDelegate;
        private IntPtr _callbackPtr;
        private IntPtr _openWindowCallbackPtr;
        private bool _isHookRunning;

        public event Action? OnOpenWindowRequested;

        public event Action<bool>? OnVietnameseModeChanged;
        public event Action<string>? OnError;

        public EngineService()
        {
            _engine = new EngineWrapper();
            InitializeHook();
        }

        private void InitializeHook()
        {
            _callbackDelegate = new ToggleCallbackDelegate(HandleToggleCallback);
            _callbackPtr = Marshal.GetFunctionPointerForDelegate(_callbackDelegate);
            EngineWrapper.vnkey_set_toggle_callback(_callbackPtr);

            _openWindowCallbackDelegate = new OpenWindowCallbackDelegate(HandleOpenWindowCallback);
            _openWindowCallbackPtr = Marshal.GetFunctionPointerForDelegate(_openWindowCallbackDelegate);
            EngineWrapper.vnkey_set_open_window_callback(_openWindowCallbackPtr);
        }

        private void HandleOpenWindowCallback()
        {
            OnOpenWindowRequested?.Invoke();
        }

        private void HandleToggleCallback(bool isEnabled)
        {
            OnVietnameseModeChanged?.Invoke(isEnabled);
        }

        public void StartHook()
        {
            if (!_isHookRunning)
            {
                EngineWrapper.vnkey_hook_start();
                _isHookRunning = true;
            }
        }

        public void StopHook()
        {
            if (_isHookRunning)
            {
                EngineWrapper.vnkey_hook_stop();
                _isHookRunning = false;
            }
        }

        public void SetMode(InputMode mode) => ExecuteSafe(() => _engine.SetMode(mode));

        public void SetVietnameseMode(bool enabled) => ExecuteSafe(() => _engine.SetVietnameseMode(enabled));

        public void Reset() => ExecuteSafe(() => _engine.Reset());

        public void SetConfig(string jsonConfig) => ExecuteSafe(() => EngineWrapper.vnkey_global_set_config_json(jsonConfig));

        public void SetShorthand(string jsonShorthand) => ExecuteSafe(() => EngineWrapper.vnkey_global_set_shorthand_json(jsonShorthand));

        public void LoadDictionary(string path) => ExecuteSafe(() => EngineWrapper.vnkey_global_load_dictionary(path));

        public string GetDiagnosticInfo() 
        {
            try { return _engine.GetDiagnosticInfo(); }
            catch (Exception ex) { OnError?.Invoke($"Engine Error: {ex.Message}"); return string.Empty; }
        }

        private void ExecuteSafe(Action action)
        {
            try
            {
                action();
            }
            catch (Exception ex)
            {
                OnError?.Invoke($"Engine Error: {ex.Message}");
            }
        }

        public void Dispose()
        {
            StopHook();
            _engine.Dispose();
        }
    }
}
