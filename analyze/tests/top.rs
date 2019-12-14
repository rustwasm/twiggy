mod top_tests {
    use twiggy_analyze::top;
    use twiggy_ir::ItemsBuilder;
    use twiggy_ir::{Id, Item, Misc};
    use twiggy_opt::Top as Options;
    use twiggy_traits::Error;

    #[test]
    fn empty() -> Result<(), Error> {
        let builder = ItemsBuilder::new(0);
        let mut items = builder.finish();
        let opts = Options::default();
        let top_items = top(&mut items, &opts)?;
        assert_eq!(top_items.items(), []);
        Ok(())
    }

    #[test]
    fn one_item() -> Result<(), Error> {
        let mut builder = ItemsBuilder::new(0);
        builder.add_item(Item::new(
            Id::entry(0, 0),
            "0_10".to_string(),
            10,
            Misc::new(),
        ));
        let mut items = builder.finish();
        let opts = Options::default();
        let top_items = top(&mut items, &opts)?;
        assert_eq!(top_items.items(), [Id::entry(0, 0)]);
        Ok(())
    }

    #[test]
    fn sorts_items() -> Result<(), Error> {
        let mut builder = ItemsBuilder::new(0);
        builder.add_item(Item::new(
            Id::entry(0, 0),
            "size 10".to_string(),
            10,
            Misc::new(),
        ));
        builder.add_item(Item::new(
            Id::entry(0, 1),
            "size 20".to_string(),
            20,
            Misc::new(),
        ));
        builder.add_item(Item::new(
            Id::entry(0, 2),
            "size 1".to_string(),
            1,
            Misc::new(),
        ));
        let mut items = builder.finish();
        let opts = Options::default();
        let top_items = top(&mut items, &opts)?;
        assert_eq!(
            top_items.items(),
            [Id::entry(0, 1), Id::entry(0, 0), Id::entry(0, 2),]
        );
        Ok(())
    }
}
