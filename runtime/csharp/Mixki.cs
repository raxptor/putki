using System;
using System.Collections.Generic;
using System.Text;

namespace Mixki
{
    public static class Parse
    {
        public static float Float(Dictionary<string, object> dict, string name, float def)
        {
            object tmp;
            if (dict.TryGetValue(name, out tmp))
                return float.Parse(tmp.ToString(), System.Globalization.CultureInfo.InvariantCulture);
            return def;
        }
        public static int Int(Dictionary<string, object> dict, string name, int def)
        {
            object tmp;
            if (dict.TryGetValue(name, out tmp))
                return int.Parse(tmp.ToString());
            return def;
        }
        public static int Int(object value, int def)
        {
            return int.Parse(value.ToString());
        }
        public static bool Bool(Dictionary<string, object> dict, string name, bool def)
        {
            object tmp;
            if (dict.TryGetValue(name, out tmp))
                return (tmp.ToString() == "True" || tmp.ToString() == "true" || tmp.ToString() == "1");
            return def;
        }
        public static float Float(object value, float def)
        {
            return float.Parse(value.ToString(), System.Globalization.CultureInfo.InvariantCulture);
        }
        public static string String(Dictionary<string, object> dict, string name, string def)
        {
            object tmp;
            if (dict.TryGetValue(name, out tmp))
                return tmp.ToString();
            return def;
        }
    }
}
