# usr/bin/sh
if [[ "$OSTYPE" == "darwin"* ]]; then
    curl -s https://api.github.com/repos/fifthtry/fpm/releases/latest | grep ".*\/releases\/download\/.*\/fpm_macos.*" | cut -d : -f 2,3 | xargs -I % curl -O -J -L %
    mv fpm_macos_x86_64 /usr/local/bin/fpm
else
    curl -s https://api.github.com/repos/fifthtry/fpm/releases/latest | grep ".*\/releases\/download\/.*\/fpm_linux.*" | cut -d : -f 2,3 | xargs -I % curl -O -J -L %
    mv fpm_linux_musl_x86_64 /usr/local/bin/fpm
    mv fpm_linux_musl_x86_64.d /usr/local/bin/fpm.d
fi
chmod +x /usr/local/bin/fpm*
