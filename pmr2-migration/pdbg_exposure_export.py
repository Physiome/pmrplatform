import json
from pmr2.app.exposure.browser.browser import ExposurePort
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
        # TODO need the workflow state here
        results.append({
            "path": path,
            "workflow_state": exposure.review_state,
            "wizard_export": export,
        })

with open('/tmp/exposure_dump.json', 'w') as fd:
    json.dump(results, fd)
