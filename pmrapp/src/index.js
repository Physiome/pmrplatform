import hljs from 'highlight.js/lib/core';
import c from 'highlight.js/lib/languages/c';
import fortran from 'highlight.js/lib/languages/fortran';
import matlab from 'highlight.js/lib/languages/matlab';
import python from 'highlight.js/lib/languages/python';
hljs.registerLanguage('c', c);
hljs.registerLanguage('fortran', fortran);
hljs.registerLanguage('matlab', matlab);
hljs.registerLanguage('python', python);

export { hljs };
