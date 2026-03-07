using VNKey.Windows.Models;

namespace VNKey.Windows.Services
{
    public class ConfigService : IConfigService
    {
        private AppConfig _config;

        public AppConfig Config => _config;

        public ConfigService()
        {
            _config = AppConfig.Load();
        }

        public void Load()
        {
            _config = AppConfig.Load();
        }

        public void Save()
        {
            _config.Save();
        }

        public void SyncWithUi(AppConfig updatedConfig)
        {
            _config = updatedConfig;
            Save();
        }

        public void ResetToDefault()
        {
            _config = new AppConfig();
            Save();
        }
    }
}
