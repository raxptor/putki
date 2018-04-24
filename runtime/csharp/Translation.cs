using System;
using System.IO;
using System.Collections.Generic;
using Putki;

namespace Putki
{
    public interface Translation
    {
        string Translate(string text, string category);
        string Translate(string text, string category, params object[] args);
    }
}
