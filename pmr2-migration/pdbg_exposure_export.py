import json
from pmr2.app.exposure.browser.browser import ExposurePort

def timestamp(dt):
    if dt:
        return int(dt.timeTime())

results = []
records = app.pmr.portal_catalog(portal_type='Workspace', review_state='published')
for record in records:
    exposure_workspace = record.getPath()
    exposures = app.pmr.portal_catalog(
        portal_type='Exposure',
        review_state='published',
        pmr2_exposure_workspace=exposure_workspace,
    )
    for exposure in exposures:
        path = exposure.getPath()
        obj = exposure.getObject()
        exporter = ExposurePort(obj, None)
        export = list(exporter.export())
        results.append({
            "path": path,
            "workflow_state": exposure.review_state,
            "wizard_export": export,

            "creation_date": timestamp(obj.creation_date),
            "effective_date": timestamp(obj.effective_date),
        })

with open('/tmp/exposure_dump.json', 'w') as fd:
    json.dump(results, fd)
