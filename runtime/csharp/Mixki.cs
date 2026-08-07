using System;
using System.Collections.Generic;
using System.Text;

namespace Mixki
{
    /// Bad hand-authored source data, e.g. an enum value naming no member.
    /// Lives here rather than with the package reader so that generated mixki
    /// code stays independent of Package.cs, which mixki-only projects do not
    /// necessarily compile.
    public class ParseException : Exception
    {
        public ParseException(string message) : base(message) { }
    }

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
                return Bool(tmp, def);
            return def;
        }
        // The generator emits this overload for bool[] elements, same as Int and Float.
        public static bool Bool(object value, bool def)
        {
            string s = value.ToString();
            return (s == "True" || s == "true" || s == "1");
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
