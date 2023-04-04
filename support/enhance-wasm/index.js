const rust = import('./pkg');
rust
    .then(async m => {
        console.log('result ' + Date.now() + ' = ' + m.test("sss"))
        console.log('result ' + Date.now() + '  = ' + m.test("sss"))
        console.log('result ' + Date.now() + '  = ' + m.test("sss"))
    })
    .catch(console.error);