using System.Collections.Generic;
using VNKey.Windows.Models;

namespace VNKey.Windows.Services
{
    public interface IConfigService
    {
        AppConfig Config { get; }
        void Load();
        void Save();
        void SyncWithUi(AppConfig updatedConfig);
        void ResetToDefault();
    }
}
