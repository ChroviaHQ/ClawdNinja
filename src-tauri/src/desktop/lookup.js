function run(argv) {
  ObjC.import('AppKit');
  const id = argv[0];
  const url = $.NSWorkspace.sharedWorkspace.URLForApplicationWithBundleIdentifier(id);
  if (url.isNil() || !$.NSFileManager.defaultManager.fileExistsAtPath(url.path)) {
    return 'missing';
  }
  const apps = $.NSRunningApplication.runningApplicationsWithBundleIdentifier(id).js;
  return apps.length ? apps.map(app => app.processIdentifier).join(',') : 'ready';
}
