#!/bin/sh

srcdirs="src"
uidirs="data/resources/ui"

# find source files that contain gettext keywords
rust_files="$(grep -lR --include='*.rs' 'gettext\b' $srcdirs)"

# find ui files that contain translatable string
ui_files="$(grep -lRi --include='*.ui' 'translatable="[ty1]' $uidirs)"

files="$rust_files $ui_files"

# Test 1: find all files that are missing from POTFILES
missing="$(for f in $files; do ! grep -q "^$f$" po/POTFILES && echo "$f"; done)"
if [ ${#missing} -ne 0 ]; then
  echo >&2 "The following files are missing from po/POTFILES:"
  for f in ${missing}; do
    echo "  $f" >&2
  done
  echo >&2
  exit 1
fi

# Test 2: find potentially translatable properties with the translatable attribute
translatable_properties="$(grep -Roh --include='*.ui' 'name=\".*\" translatable=\".*\">' $uidirs | sed 's/^name=\"\([^\"]*\)\".*/\1/' | sort | uniq)"
missed_properties=0

for p in ${translatable_properties}; do
  missed_translatable="$(grep -lRP --include='*.ui' "<property name=\"$p\"(?! translatable=\"(no|yes)\")")"
  if [ ${#missed_translatable} -ne 0 ]; then
    missed_properties=$(expr $missed_properties + 1)

    echo >&2 "The following files contain a <property name=\"$p\"> without translatable=\"[yes|no]\":"
    for f in ${missed_translatable}; do
      echo "  $f" >&2
    done
    echo >&2
  fi
done

if [ ${missed_properties} -ne 0 ]; then
  exit 1
fi
