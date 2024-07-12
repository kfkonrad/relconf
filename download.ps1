#!/usr/bin/env sh

# this is a universal shell script running in both Bash/ZSH and powershell
# see https://stackoverflow.com/a/67292076/5921480 for details

echo --% >/dev/null;: ' | out-null
<#'


#
# sh part
#
os=$(uname -s)
arch=$(uname -m)
archive_name=""
version=""

if test "${os}" = "Darwin"
then
  if test "${arch}" = "arm64"
  then
    archive_name=aarch64-apple-darwin.tar.gz
  else
    archive_name=x86_64-apple-darwin.tar.gz
  fi
fi

if test "${os}" = "Linux"
then
  case $arch in
    aarch64)
      archive_name=aarch64-unknown-linux-musl.tar.gz
      ;;

    x86_64)
      archive_name=x86_64-unknown-linux-musl.tar.gz
      ;;

    armv7l)
      archive_name=armv7-unknown-linux-musleabi.tar.gz
      ;;

    *)
      ;;
  esac
fi

if test -z "$archive_name"
then
  echo unknown os and architecture combination "$os" "$arch", aborting.
  echo Check https://github.com/kfkonrad/relconf/releases
  echo for precompiled binaries or install using cargo install standard-readme.
  exit 1
fi

if which curl > /dev/null 2> /dev/null
then
  version=$(curl -s https://api.github.com/repos/kfkonrad/relconf/releases/latest | grep '"html_url": "https://github.com/kfkonrad/relconf/releases/tag/' | sed 's,^.*"html_url": "https://github.com/kfkonrad/relconf/releases/tag/,,;s/",.*$//')
    curl -sL -o "${archive_name}" "https://github.com/kfkonrad/relconf/releases/download/${version}/${archive_name}"
else
  if which wget > /dev/null 2> /dev/null
  then
    version=$(wget -qO- https://api.github.com/repos/kfkonrad/relconf/releases/latest | grep '"html_url": "https://github.com/kfkonrad/relconf/releases/tag/' | sed 's,^.*"html_url": "https://github.com/kfkonrad/relconf/releases/tag/,,;s/",.*$//')
    wget "https://github.com/kfkonrad/relconf/releases/download/${version}/${archive_name}"
  else
    echo error: neither curl nor wget found. Install wget or curl to continue
    echo or download directly from https://github.com/kfkonrad/relconf/releases/download/${version}/${archive_name}
    exit 1
  fi
fi

tar xf $archive_name
rm $archive_name
echo Download complete. Add standard-readme to a folder in your PATH to use from anywhere or start using it with ./standard-readme
exit #>


#
# powershell part
#

$response = Invoke-WebRequest -Uri "https://api.github.com/repos/kfkonrad/relconf/releases/latest" -UseBasicParsing
$json = $response.Content | ConvertFrom-Json
$version = $json.html_url -replace '^https://github.com/kfkonrad/relconf/releases/tag/', '' -replace '/$'
$archive_name = "x86_64-pc-windows-gnu.zip"
Invoke-WebRequest -Uri "https://github.com/kfkonrad/relconf/releases/download/$version/$archive_name" -OutFile $archive_name
Expand-Archive -Path $archive_name -DestinationPath .
del $archive_name
echo "Download complete. Add standard-readme to a folder in your PATH to use from anywhere or start using it with ./standard-readme"
