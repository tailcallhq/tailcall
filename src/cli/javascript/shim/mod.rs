use rquickjs::{Ctx, Object, Function, Result};

pub struct QuickJs;

impl QuickJs {
    pub fn new<'js>(ctx: Ctx<'js>) -> Object<'js> {
        let mut core = Object::new(ctx).unwrap();
        core.set("print", Function::new(ctx, print));

        let qjs = Object::new(ctx).unwrap();
        qjs.set("core", core);
        qjs
    }
}

fn print<'js>(ctx: Ctx<'js>, message: String, is_error: bool) -> Result<()> {
    Ok(())
}

