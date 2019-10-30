if [[ $PUBLISH_DOCS ]]; then
  cargo doc
  echo '<meta http-equiv="refresh content=0;url=new_url/index.html">' > target/doc/index.html
  echo 'git clone https://github.com/davisp/ghp-import.git && ./ghp-import/ghp_import.py -n -p -f -m "Documentation upload" -r https://"$GH_TOKEN"@github.com/"$TRAVIS_REPO_SLUG.git" target/doc && echo "Uploaded documentation"'
fi