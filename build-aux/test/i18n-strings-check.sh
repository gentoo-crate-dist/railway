#!/bin/sh

srcdirs="src"
uidirs="data/resources/ui"

# find source files that contain gettext keywords
rust_files="$(grep -lR --include='*.rs' 'gettext\b' $srcdirs)"

# find ui
blp_files="$(grep -lRi --include='*.blp' '_("' $uidirs)"

files="$rust_files $blp_files"

exitval=0

# Test 1: find all files that are missing from POTFILES
missing="$(for f in $files; do ! grep -q "^$f$" po/POTFILES && echo "$f"; done)"
if [ ${#missing} -ne 0 ]; then
  echo >&2 "The following files are missing from po/POTFILES:"
  for f in ${missing}; do
    echo "  $f" >&2
  done
  echo >&2
  exitval=1
fi

# Test 2: find potentially translatable properties without _("…") that are not marked as // not translated
translatable_properties="$(grep -Roh --include='*.blp' '[a-zA-Z][a-zA-Z0-9_-]*\s*:\s*_(' $uidirs | sed 's/^\([a-zA-Z][a-zA-Z0-9_-]*\).*/\1/' | sort | uniq)"

for p in ${translatable_properties}; do
  missed_translatable="$(grep -lRP --include='*.blp' "$p\s*:\s*(\".*\"|'.*')\s*;(?! // not translated)" $uidirs)"
  if [ ${#missed_translatable} -ne 0 ]; then
    exitval=1

    echo >&2 "The following files use the property $p without \`_("…")\` or a \`// not translated\` comment:"
    for f in ${missed_translatable}; do
      echo "  $f" >&2
    done
    echo >&2
  fi
done

# Test 3: find all blueprint files which use a gettext qualifier that will not be picked up by gettext by default.
# The gettext qualifier _('.*') is not picked up by gettext, one should use _("") instead (double quotes instead of single quotes).
blp_files_with_wrong_gettext="$(grep -lRi --include='*.blp' "_('.*')" $uidirs)"
if [ ${#blp_files_with_wrong_gettext} -ne 0 ]; then
  echo >&2 "The following blueprint files use the wrong gettext quantifier (\`_('…')\` instead of \`_("…")\`):"
  for f in ${blp_files_with_wrong_gettext}; do
    echo "  $f" >&2
  done
  echo >&2
  exitval=1
fi

exit $exitval
