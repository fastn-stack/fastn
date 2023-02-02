import os
import sys


def replace_in_ftd_files(root_dir):
    for subdir, dirs, files in os.walk(root_dir):
        for dir in dirs:
            if dir == 'FPM':
                old_dir_path = os.path.join(subdir, dir)
                new_dir_path = os.path.join(subdir, 'FASTN')
                os.rename(old_dir_path, new_dir_path)
        for file in files:
            if file == 'deploy.yml':
                file_path = os.path.join(subdir, file)
                with open(file_path, 'r') as f:
                    contents = f.read()
                contents = contents.replace('install-fpm', 'install') \
                    .replace('--edition=2022', '') \
                    .replace('fpm', 'fastn') \
                    .replace('FPM', 'FASTN') \
                    .replace('fpm.dev', 'fastn.io') \
                    .replace('https://raw.githubusercontent.com/FifthTry', 'https://raw.githubusercontent.com/ftd-lang')
                with open(file_path, 'w') as f:
                    f.write(contents)

            if file.endswith('.ftd'):
                file_path = os.path.join(subdir, file)
                with open(file_path, 'r') as f:
                    contents = f.read()
                contents = contents.replace('fpm', 'fastn')\
                    .replace('FPM', 'FASTN')\
                    .replace('fpm.dev', 'fastn.io')
                with open(file_path, 'w') as f:
                    f.write(contents)

            if file == 'FPM.ftd':
                old_file_path = os.path.join(subdir, file)
                new_file_path = os.path.join(subdir, 'FASTN.ftd')
                os.rename(old_file_path, new_file_path)
                continue

            if 'fpm' in file:
                old_file_path = os.path.join(subdir, file)
                new_file = file.replace('fpm', 'fastn')
                new_file_path = os.path.join(subdir, new_file)
                os.rename(old_file_path, new_file_path)

            if 'FPM' in file:
                old_file_path = os.path.join(subdir, file)
                file.replace('FPM', 'FASTN')
                new_file_path = os.path.join(subdir, new_file)
                os.rename(old_file_path, new_file_path)


if __name__ == "__main__":
    if sys.argv[1]:
        root_directory = sys.argv[1]
        replace_in_ftd_files(root_directory)
    else:
        print("write like: `python rename.py <package-path>`")


# How To Run this
# Let's say we have package package-doc and this file in the same directory
# python rename.py package-doc

