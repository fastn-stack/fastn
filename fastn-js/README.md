# How to Format JavaScript Files

JavaScript formatting is tested using [`dprint-check-action`](https://github.com/marketplace/actions/dprint-check-action). 
It utilizes the [dprint-prettier-plugin](https://dprint.dev/plugins/prettier/). 

To format the `fastn-js` files, you can install [dprint](https://dprint.dev/install/) on your system and execute the following command from the root of the project:

```bash
dprint fmt --config=.github/dprint-ci.json
```

Pull Request: [fastn-stack/fastn#1661](https://github.com/fastn-stack/fastn/pull/1661)
