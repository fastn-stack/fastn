# usr/bin/sh

if [[ $* == *--pre-release* ]]; then
    URL="https://api.github.com/repos/fifthtry/fpm/releases"
else
    URL="https://api.github.com/repos/fifthtry/fpm/releases/latest"
fi

if [[ "$OSTYPE" == "darwin"* ]]; then
    curl -s $URL | grep ".*\/releases\/download\/.*\/fpm_macos.*" | head -1 | cut -d : -f 2,3 | xargs -I % curl -O -J -L %
    mv fpm_macos_x86_64 /usr/local/bin/fpm
else
    curl -s $URL | grep ".*\/releases\/download\/.*\/fpm_linux.*" | head -2 | cut -d : -f 2,3 | xargs -I % curl -O -J -L %
    mv fpm_linux_musl_x86_64 /usr/local/bin/fpm
    mv fpm_linux_musl_x86_64.d /usr/local/bin/fpm.d
fi
chmod +x /usr/local/bin/fpm*
