use super::*;

#[test]
fn test() {
    println!("{:x?}", Fr::from_str_vartime("10944121435919637611123202872628637544274182200208017171849102093287904247808").unwrap().into_raw())
}

#[test]
fn test_add_same_point() {
    let p: PointProjective = PointProjective {
        x: Fr::from_str_vartime(
            "17777552123799933955779906779655732241715742912184938656739573121738514868268",
        )
        .unwrap(),
        y: Fr::from_str_vartime(
            "2626589144620713026669568689430873010625803728049924121243784502389097019475",
        )
        .unwrap(),
        z: Fr::ONE,
    };
    let res = (p + p).affine();
    assert_eq!(
        res.x,
        Fr::from_str_vartime(
            "6890855772600357754907169075114257697580319025794532037257385534741338397365"
        )
        .unwrap()
    );
    assert_eq!(
        res.y,
        Fr::from_str_vartime(
            "4338620300185947561074059802482547481416142213883829469920100239455078257889"
        )
        .unwrap()
    );
}

#[test]
fn test_add_different_points() {
    let p: PointProjective = PointProjective {
        x: Fr::from_str_vartime(
            "17777552123799933955779906779655732241715742912184938656739573121738514868268",
        )
        .unwrap(),
        y: Fr::from_str_vartime(
            "2626589144620713026669568689430873010625803728049924121243784502389097019475",
        )
        .unwrap(),
        z: Fr::ONE,
    };
    let q: PointProjective = PointProjective {
        x: Fr::from_str_vartime(
            "16540640123574156134436876038791482806971768689494387082833631921987005038935",
        )
        .unwrap(),
        y: Fr::from_str_vartime(
            "20819045374670962167435360035096875258406992893633759881276124905556507972311",
        )
        .unwrap(),
        z: Fr::ONE,
    };
    let res = (p + q).affine();
    assert_eq!(
        res.x,
        Fr::from_str_vartime(
            "7916061937171219682591368294088513039687205273691143098332585753343424131937"
        )
        .unwrap()
    );
    assert_eq!(
        res.y,
        Fr::from_str_vartime(
            "14035240266687799601661095864649209771790948434046947201833777492504781204499"
        )
        .unwrap()
    );
}

#[test]
fn test_mul_scalar() {
    let p: Point = Point {
        x: Fr::from_str_vartime(
            "17777552123799933955779906779655732241715742912184938656739573121738514868268",
        )
        .unwrap(),
        y: Fr::from_str_vartime(
            "2626589144620713026669568689430873010625803728049924121243784502389097019475",
        )
        .unwrap(),
    };
    let res_m = p.mul_scalar(&Fr::from(3));
    let res_a = p.projective() + p.projective();
    let res_a = (res_a + p.projective()).affine();
    assert_eq!(res_m.x, res_a.x);
    assert_eq!(
        res_m.x,
        Fr::from_str_vartime(
            "19372461775513343691590086534037741906533799473648040012278229434133483800898"
        )
        .unwrap()
    );
    assert_eq!(
        res_m.y,
        Fr::from_str_vartime(
            "9458658722007214007257525444427903161243386465067105737478306991484593958249"
        )
        .unwrap()
    );

    let n = Fr::from_str_vartime(
        "14035240266687799601661095864649209771790948434046947201833777492504781204499",
    )
    .unwrap();
    let res2 = p.mul_scalar(&n);
    assert_eq!(
        res2.x,
        Fr::from_str_vartime(
            "17070357974431721403481313912716834497662307308519659060910483826664480189605"
        )
        .unwrap()
    );
    assert_eq!(
        res2.y,
        Fr::from_str_vartime(
            "4014745322800118607127020275658861516666525056516280575712425373174125159339"
        )
        .unwrap()
    );
}

#[test]
fn test_point_compress_decompress() {
    let p: Point = Point {
        x: Fr::from_str_vartime(
            "17777552123799933955779906779655732241715742912184938656739573121738514868268",
        )
        .unwrap(),
        y: Fr::from_str_vartime(
            "2626589144620713026669568689430873010625803728049924121243784502389097019475",
        )
        .unwrap(),
    };
    let p_comp = p.compress();
    assert_eq!(
        hex::encode(p_comp),
        "53b81ed5bffe9545b54016234682e7b2f699bd42a5e9eae27ff4051bc698ce85"
    );
    let p2 = Point::decompress(p_comp).unwrap();
    assert_eq!(p.x, p2.x);
    assert_eq!(p.y, p2.y);
}

#[test]
fn test_point_decompress0() {
    let y_bytes_raw =
        hex::decode("b5328f8791d48f20bec6e481d91c7ada235f1facf22547901c18656b6c3e042f")
            .unwrap();
    let mut y_bytes: [u8; 32] = [0; 32];
    y_bytes.copy_from_slice(&y_bytes_raw);
    let p = Point::decompress(y_bytes).unwrap();

    let expected_px_raw =
        hex::decode("b86cc8d9c97daef0afe1a4753c54fb2d8a530dc74c7eee4e72b3fdf2496d2113")
            .unwrap();
    let mut e_px_bytes: [u8; 32] = [0; 32];
    e_px_bytes.copy_from_slice(&expected_px_raw);
    let expected_px: Fr = Fr::from_repr_vartime(FrRepr(e_px_bytes)).unwrap();
    assert_eq!(&p.x, &expected_px);
}

#[test]
fn test_point_decompress1() {
    let y_bytes_raw =
        hex::decode("70552d3ff548e09266ded29b33ce75139672b062b02aa66bb0d9247ffecf1d0b")
            .unwrap();
    let mut y_bytes: [u8; 32] = [0; 32];
    y_bytes.copy_from_slice(&y_bytes_raw);
    let p = Point::decompress(y_bytes).unwrap();

    let expected_px_raw =
        hex::decode("30f1635ba7d56f9cb32c3ffbe6dca508a68c7f43936af11a23c785ce98cb3404")
            .unwrap();
    let mut e_px_bytes: [u8; 32] = [0; 32];
    e_px_bytes.copy_from_slice(&expected_px_raw);
    let expected_px: Fr = Fr::from_repr_vartime(FrRepr(e_px_bytes)).unwrap();
    assert_eq!(&p.x, &expected_px);
}