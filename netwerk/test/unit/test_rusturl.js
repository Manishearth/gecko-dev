// Tests various operations on nsIStandardURI using rust-url
const StandardURL = Components.Constructor("@mozilla.org/network/standard-url;1",
                                           "nsIStandardURL",
                                           "init");
const nsIStandardURL = Components.interfaces.nsIStandardURL;

const prefs = Cc["@mozilla.org/preferences-service;1"]
              .getService(Ci.nsIPrefBranch);

function stringToURL(str) {
  return (new StandardURL(nsIStandardURL.URLTYPE_AUTHORITY, 80,
       str, "UTF-8", null))
         .QueryInterface(Components.interfaces.nsIURL);
}

function pairToURLs(pair) {
  do_check_eq(pair.length, 2);
  return pair.map(stringToURL);
}

function test_rust() {
  var url = stringToURL("http://user:password@example.com/path/filename.ext?query=good&param=better#hash");
  equal(url.spec, "http://user:password@example.com/path/filename.ext?query=good&param=better#hash");

  var url = stringToURL("http://example.com");
  equal(url.spec, "http://example.com/");

  equal(url.hasRef, false);
  url.ref = "#a";
  equal(url.ref, "a");
  equal(url.hasRef, true);
  url.ref = "#";

  equal(url.ref, "");
  equal(url.spec, "http://example.com/#")
  equal(url.hasRef, true);

  url.ref = "";
  equal(url.ref, "");
  equal(url.spec, "http://example.com/")
  equal(url.hasRef, false);

  url.username = "u";
  equal(url.username, "u");
  url.username = "";
  equal(url.username, "");

  url.username = "u";
  url.password = "p";
  equal(url.password, "p");
  url.password = "";
  equal(url.password, "");

  equal(url.userPass, "u");
  url.password = "p";
  equal(url.userPass, "u:p");
  url.userPass = "user:password";
  equal(url.username, "user");
  equal(url.password, "password");
  equal(url.userPass, "user:password");
  url.userPass = "";
  equal(url.userPass, "");

  url.scheme = "https";
  equal(url.scheme, "https");
  url.scheme = "http";
  equal(url.scheme, "http");

  url.host = "bla";
  equal(url.host, "bla");
  url.port = 10;
  equal(url.port, 10);
  equal(url.hostPort, "bla:10");

  url.path = "/abc/def";
  equal(url.path, "/abc/def");

  url = stringToURL("resource://gre-resources/hiddenWindow.html");
  equal(url.spec, "resource://gre-resources/hiddenWindow.html");

  url = stringToURL("http://host/directory/fileBaseName.fileExtension?query#ref");
  equal(url.filePath, "/directory/fileBaseName.fileExtension");
  equal(url.directory, "/directory/");
  equal(url.fileName, "fileBaseName.fileExtension");
  equal(url.fileBaseName, "fileBaseName");
  equal(url.fileExtension, "fileExtension");

  url = stringToURL("http://host/file?query#ref");
  equal(url.filePath, "/file");
  equal(url.directory, "/");
  equal(url.fileName, "file");
  equal(url.fileBaseName, "file");
  equal(url.fileExtension, "");
}

function run_test()
{
  prefs.setBoolPref("network.standard-url.use-rust", v.prefValue);
  test_rust();
  prefs.clearUserPref("network.standard-url.use-rust");
}
