using System;
using VNKey.Windows.Models;

namespace VNKey.Windows.Services
{
    public interface IEngineService
    {
        void StartHook();
        void StopHook();
        void SetMode(InputMode mode);
        void SetVietnameseMode(bool enabled);
        void Reset();
        void SetConfig(string jsonConfig);
        void SetShorthand(string jsonShorthand);
        void LoadDictionary(string path);
        string GetDiagnosticInfo();
        
        event Action<bool>? OnVietnameseModeChanged;
        event Action<string>? OnError;
    }
}
