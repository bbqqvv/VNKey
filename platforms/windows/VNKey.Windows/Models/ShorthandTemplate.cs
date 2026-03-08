using System.Collections.Generic;

namespace VNKey.Windows.Models
{
    public class ShorthandTemplate
    {
        public string Name { get; set; }
        public string Icon { get; set; }
        public string Description { get; set; }
        public List<ShorthandItem> Items { get; set; } = new List<ShorthandItem>();
    }
}
