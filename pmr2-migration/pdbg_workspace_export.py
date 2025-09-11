import json
results = []
records = app.pmr.portal_catalog(portal_type='Workspace', review_state='published')
for record in records:
    path = record.getPath()
    result = {
        # cannot use id naively as alias due to `/w/{user}/`
        # 'alias': record.id
        # instead, drop site prefix and sub out strings
        'alias': path
            .replace('/pmr/', '')
            # this should maintain the id for default case
            .replace('workspace/', '')
            # drop the prefix for users and replace sep with `-`
            .replace('w/', '')
            .replace('/', '-'),
        'description': record.Title,
        'path': path,
        'url': path.replace('/pmr/', 'https://models.physiomeproject.org/') + '/',
        'workflow_state': record.review_state,
    }
    results.append(result)

with open('/tmp/workspace_dump.json', 'w') as fd:
    json.dump(results, fd)
