import os

directory = os.getcwd()

for filename in os.listdir(directory):
    print(filename)
    if not filename.endswith(".ftd"):
        continue
    f = open(os.path.join(directory, filename), 'r')
    text = f.read()
    t = text.replace("$fpm.design.type.", "$fpm.type.")\
        .replace("$fpm.design.color.brand.on-region", "$fpm.color.main.text")\
        .replace("$fpm.design.color.brand.region", "$fpm.color.main.background.base")\
        .replace("$fpm.design.color.brand.primary-action", "$fpm.color.main.cta-primary.base")\
        .replace("$fpm.design.color.brand.on-primary-action", "$fpm.color.main.cta-primary.text")\
        .replace("$fpm.design.color.brand.secondary-action", "$fpm.color.main.cta-secondary.base")\
        .replace("$fpm.design.color.brand.on-secondary-action", "$fpm.color.main.cta-secondary.text")\
        .replace("$fpm.design.color.brand.error", "$fpm.color.main.error.base")\
        .replace("$fpm.design.color.brand.on-error", "$fpm.color.main.error.text") \
        .replace("$fpm.design.color.brand.success", "$fpm.color.main.success.base") \
        .replace("$fpm.design.color.brand.on-success", "$fpm.color.main.success.text") \
        .replace("$fpm.design.color.brand.warning", "$fpm.color.main.warning.base") \
        .replace("ftd.colors", "ftd.color-scheme") \
        .replace("$colors.primary-action", "$colors.cta-primary.base") \
        .replace("$colors.on-primary-action", "$colors.cta-primary.text") \
        .replace("$colors.secondary-action", "$colors.cta-secondary.base") \
        .replace("$colors.region", "$colors.background.base") \
        .replace("$colors.on-region", "$colors.text") \
        .replace("$fpm.design.color.brand", "$fpm.color.main.background.base") \
        .replace("$fpm.design.color.brand.on-warning", "$fpm.color.main.warning.text")
    f.close()
    f = open(os.path.join(directory, filename), 'w')
    f.write(t)
    f.close()
