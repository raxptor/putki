using System;
using System.IO;
using System.Collections.Generic;
using Putki;

namespace Putki
{
	[AttributeUsage(AttributeTargets.Field)]
	public class TranslatedField : Attribute
	{
		public string Category;
		public bool Plural;
	}

	public interface Translation
	{
		string Translate(string text, string category);
		string Translate(string text, string category, int plural_n);
	}
}
